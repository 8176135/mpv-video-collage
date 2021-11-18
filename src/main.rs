use std::ffi::CStr;

use libmpv::events::Event as MpvEvent;
use libmpv::{
    render::{RenderContext, RenderParam, RenderParamApiType},
    FileState, Format, Mpv,
};
use miniquad::*;
use nanorand::Rng;

mod windows_specific;

const IGNORED_ITEMS: &[&str] = include!("../included_consts/ignored_items.rs");
const WALK_DIR_PATH: &str = include_str!("../included_consts/look_path.txt");

const VIDEO_WIDTH: usize = 1280;
const VIDEO_HEIGHT: usize = 720;

struct LocalPath {
    stuff: Vec<String>,
}

trait RandomPathGenerator {
    fn get_random(&self) -> String;
}

impl RandomPathGenerator for LocalPath {
    fn get_random(&self) -> String {
        self.stuff[nanorand::tls_rng().generate_range(0..self.stuff.len())].clone()
    }
}

// struct RemotePath {
//     agent: Agent,
// }

// impl RandomPathGenerator for RemotePath {
//     fn get_random(&self) -> String {
//         let res = self
//             .agent
//             .post("https://lan.8176135.xyz:8000/api/random-temp/")
//             .send_json(serde_json::to_value(Configuration::default()).unwrap())
//             .unwrap();

//         let data = res.into_string().unwrap();

//         return format!("https://lan.8176135.xyz:8000/temp-files/{}", data);
//     }
// }

#[repr(C)]
struct Vec2 {
    x: f32,
    y: f32,
}
#[repr(C)]
struct Vertex {
    pos: Vec2,
    uv: Vec2,
}

struct ItemHolder {
    mpv: Mpv,
    position: (usize, usize),
    size: (i32, i32),
    render_context: RenderContext,
}

struct Stage<RPG: RandomPathGenerator> {
    pipeline: Pipeline,
    bindings: Bindings,
    texture_buffer: Vec<u8>,
    mpv_storage: Vec<ItemHolder>,
    // last_time: std::time::Instant,
    path_collection: RPG,
}

impl<T: RandomPathGenerator> Stage<T> {
    pub fn new(ctx: &mut Context, path_collection: T, count: (usize, usize)) -> Stage<T> {
        #[rustfmt::skip]
        let vertices: [Vertex; 4] = [
            Vertex { pos : Vec2 { x: -0.5, y: -0.5 }, uv: Vec2 { x: 0., y: 0. } },
            Vertex { pos : Vec2 { x:  0.5, y: -0.5 }, uv: Vec2 { x: 1., y: 0. } },
            Vertex { pos : Vec2 { x:  0.5, y:  0.5 }, uv: Vec2 { x: 1., y: 1. } },
            Vertex { pos : Vec2 { x: -0.5, y:  0.5 }, uv: Vec2 { x: 0., y: 1. } },
        ];
        let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);

        let pixels = vec![0; VIDEO_WIDTH * VIDEO_HEIGHT * 4 * count.0 * count.1];
        let texture = Texture::from_rgba8(ctx, 1280 * 2, 720 * 2, &pixels);

        let mut mpv_storage = Vec::new();

        {
            for y in 0..count.1 {
                for x in 0..count.0 {
                    let mut mpv = mpv_setup();
                    append_random_playlist(&mpv, &path_collection);
                    mpv_storage.push(ItemHolder {
                        position: (x * VIDEO_WIDTH, y * VIDEO_HEIGHT),
                        size: (1280, 720),
                        render_context: RenderContext::new(
                            unsafe { mpv.ctx.as_mut() },
                            vec![RenderParam::<()>::ApiType(RenderParamApiType::Software)],
                        )
                        .unwrap(),
                        mpv,
                    });
                }
            }
        }

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer: index_buffer,
            images: vec![texture],
        };

        let shader = Shader::new(ctx, shader::VERTEX, shader::FRAGMENT, shader::meta()).unwrap();

        let pipeline = Pipeline::new(
            ctx,
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("pos", VertexFormat::Float2),
                VertexAttribute::new("uv", VertexFormat::Float2),
            ],
            shader,
        );

        #[cfg(feature = "parent-hwnd")]
        windows_specific::init_parent_window();

        Stage {
            pipeline,
            bindings,
            texture_buffer: vec![0; 4 * 1280 * 720],
            mpv_storage,
            // last_time: std::time::Instant::now(),
            path_collection,
        }
    }
}

impl<T: RandomPathGenerator> EventHandler for Stage<T> {
    fn update(&mut self, _ctx: &mut Context) {
        for ele in self.mpv_storage.iter_mut() {
            if let Some(ev) = ele.mpv.event_context_mut().wait_event(0.0) {
                match ev {
                    Ok(MpvEvent::StartFile) => {
                        println!("File loaded");

                        append_random_playlist(&ele.mpv, &self.path_collection);
                    }
                    Ok(ev) => {
                        dbg!(ev);
                    }
                    Err(e) => {
                        dbg!(e);
                    }
                }
            }
        }
    }

    fn draw(&mut self, ctx: &mut Context) {
        ctx.begin_default_pass(Default::default());
        ctx.apply_pipeline(&self.pipeline);

        // if HAS_FRAME.load(Ordering::SeqCst) {
        // let now = std::time::Instant::now();
        // println!("asdasd: {:?}", now.duration_since(self.last_time));
        // self.last_time = now;
        for ele in &self.mpv_storage {
            ele.render_context
                .render_sw(
                    (1280, 720),
                    &CStr::from_bytes_with_nul(b"rgba\0").unwrap(),
                    1280 * 4,
                    &mut self.texture_buffer,
                )
                .expect("Failed to software render");
            self.bindings.images.first().unwrap().update_texture_part(
                ctx,
                ele.position.0 as i32,
                ele.position.1 as i32,
                ele.size.0,
                ele.size.1,
                &self.texture_buffer,
            );
        }

        ctx.apply_bindings(&self.bindings);
        ctx.draw(0, 6, 1);
        ctx.end_render_pass();

        ctx.commit_frame();
    }
}

fn main() {

    #[cfg(feature = "parent-hwnd")]
    {
        let mut arg_iterator = std::env::args().skip(1);
        let mut parent_hwnd: u32 = 0;
        while let Some(arg) = arg_iterator.next() {
            if arg == "-parentHWND" {
                parent_hwnd = arg_iterator.next().unwrap().parse().unwrap();
            }
        }

        windows_specific::set_parent_window(parent_hwnd);
    }

    // args.width = 2;
    // args.height = 2;

    let path_collection = LocalPath {
        stuff: walkdir::WalkDir::new(WALK_DIR_PATH)
            .into_iter()
            .filter_map(|c| c.ok())
            .filter(|c| c.file_type().is_file())
            .map(|f| f.path().to_path_buf())
            .filter(|c| {
                c.extension()
                    .map(|f| ["mp4", "webm", "mkv"].iter().any(|c| *c == f))
                    .unwrap_or(false)
            })
            .filter(|c| {
                let lossy = c.file_name().unwrap().to_string_lossy();
                !IGNORED_ITEMS.iter().any(|c| lossy.contains(c))
            })
            .map(|f| f.to_string_lossy().replace("\\", "/"))
            .collect::<Vec<String>>(),
    };

    let conf = conf::Conf {
        fullscreen: true,
        ..Default::default()
    };
    miniquad::start(conf, move |mut ctx| {
        UserData::owning(Stage::new(&mut ctx, path_collection, (3, 2)), ctx)
    });
}

mod shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 pos;
    attribute vec2 uv;
    uniform vec2 offset;
    varying lowp vec2 texcoord;
    void main() {
        gl_Position = vec4(pos * 2.0, 0, 1);
        texcoord = uv;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec2 texcoord;
    uniform sampler2D tex;
    void main() {
        lowp vec2 realcoord = vec2(texcoord.x, 1.0 - texcoord.y);
        gl_FragColor = texture2D(tex, realcoord);
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["tex".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: vec![UniformDesc::new("offset", UniformType::Float2)],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub offset: (f32, f32),
    }
}

fn mpv_setup() -> Mpv {
    let mpv = Mpv::new().unwrap();
    mpv.set_property("volume", 50).unwrap();

    mpv.set_property("panscan", 1).unwrap();
    mpv.set_property("prefetch-playlist", "yes").unwrap();
    mpv.set_property("cache", "yes").unwrap();

    let ev_ctx = mpv.event_context();

    ev_ctx.disable_deprecated_events().unwrap();
    ev_ctx.observe_property("volume", Format::Int64, 0).unwrap();
    println!("Mpv generated");

    mpv
}

fn append_random_playlist(mpv: &Mpv, path_collection: &impl RandomPathGenerator) {
    let path_to_play = path_collection.get_random();
    dbg!(&path_to_play);
    let length_in_seconds = nanorand::tls_rng().generate_range(3..=15);
    let start_pos = nanorand::tls_rng().generate_range(50..950) as f64 / 10.0;
    println!("Seconds {}", length_in_seconds);
    println!("Start percentage {}", start_pos);

    mpv.playlist_load_files(&[(
        &path_to_play,
        FileState::AppendPlay,
        Some(&format!(
            "length={},start={}%",
            length_in_seconds, start_pos
        )),
    )])
    .unwrap();
}
