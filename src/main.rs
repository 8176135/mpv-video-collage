use std::time::Duration;
use std::time::Instant;

use libmpv::FileState;
use libmpv::Format;
use libmpv::Mpv;
use nanorand::Rng;
use pixels::Pixels;
use pixels::SurfaceTexture;
use winit::dpi::PhysicalPosition;
use winit::event::ElementState;
use winit::event::MouseButton;
use winit::event::VirtualKeyCode;

use winit::window::Fullscreen;
use winit::window::WindowId;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
};

#[cfg(target_os = "linux")]
use winit::platform::unix::{WindowBuilderExtUnix, WindowExtUnix};

#[cfg(target_os = "windows")]
use winit::platform::windows::{WindowBuilderExtWindows, WindowExtWindows};

use libmpv::events::Event as MpvEvent;
use serde::Deserialize;
use serde::Serialize;
use ureq::Agent;
use argh::FromArgs;
use std::io::Read;

const IGNORED_ITEMS: &[&str] = include!("../included_consts/ignored_items.rs");
const WALK_DIR_PATH: &str = include_str!("../included_consts/look_path.txt");

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

struct RemotePath {
    agent: Agent,
}

impl RandomPathGenerator for RemotePath {
    fn get_random(&self) -> String {
        let res = self
            .agent
            .post("https://lan.8176135.xyz:8000/api/random-temp/")
            .send_json(serde_json::to_value(Configuration::default()).unwrap())
            .unwrap();

        let data = res.into_string().unwrap();

        return format!("https://lan.8176135.xyz:8000/temp-files/{}", data);
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Configuration {
    pub only_video: bool,
    pub only_non_watched: bool,
    pub length_range: (i32, i32),
    pub filter: String,
    pub show_videos_default: bool,
    pub finished_watched: bool,
    pub sort_method: SortBy,
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            only_non_watched: true,
            only_video: true,
            length_range: (0, 20),
            filter: String::new(),
            show_videos_default: false,
            finished_watched: true,
            sort_method: SortBy::Date,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub enum SortBy {
    Name,
    Date,
}

#[derive(FromArgs)]
/// Create live video collages
struct InputWindowConfig {
    #[argh(option, default = "2")]
    /// number of horizontal windows
    width: u32,

    #[argh(option, default = "2")]
    /// number of vertical windows
    height: u32,

    // #[argh(positional)]
    // /// password for server
    // password: String,
}

fn main() {

    let args: InputWindowConfig = argh::from_env();

    println!("Input password: ");
    let mut password_input = String::new();
    std::io::stdin().read_line(&mut password_input).unwrap();

    let agent = ureq::AgentBuilder::new().build();
    let res = agent
        .post("https://lan.8176135.xyz:8000/login")
        .send_form(&[(
            "password", &password_input.trim()
        )])
        .unwrap();

    dbg!(res);
    let path_collection = RemotePath { agent };
    path_collection.get_random();
    // let path_collection = LocalPath {
    //     stuff: walkdir::WalkDir::new(WALK_DIR_PATH)
    //         .into_iter()
    //         .filter_map(|c| c.ok())
    //         .filter(|c| c.file_type().is_file())
    //         .map(|f| f.path().to_path_buf())
    //         .filter(|c| {
    //             c.extension()
    //                 .map(|f| ["mp4", "webm", "mkv"].iter().any(|c| *c == f))
    //                 .unwrap_or(false)
    //         })
    //         .filter(|c| {
    //             let lossy = c.file_name().unwrap().to_string_lossy();
    //             !IGNORED_ITEMS.iter().any(|c| lossy.contains(c))
    //         })
    //         .map(|f| f.to_string_lossy().replace("\\", "/"))
    //         .collect::<Vec<String>>()
    // };

    spawn_mpv_window(path_collection, &args);
}

fn spawn_mpv_window(path_collection: impl RandomPathGenerator + 'static, args: &InputWindowConfig) {
    let event_loop = EventLoop::new();
    let window_one = WindowBuilder::new()
        .with_maximized(true)
        .build(&event_loop)
        .unwrap();

    // let window_two = WindowBuilder::new()
    //     .with_maximized(true)
    //     .build(&event_loop)
    //     .unwrap();

    // let mut pixels = {
    //     let window_size = window.inner_size();
    //     let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
    //     Pixels::new(100, 100, surface_texture).unwrap()
    // };

    let mut stuff_holder = Vec::new();

    let max_width = args.width;
    let max_height = args.height;
    for w in 0..max_width {
        for h in 0..max_height {
            #[cfg(target_os = "windows")]
            {
                generate_sub_window(
                    &window_one,
                    &event_loop,
                    &mut stuff_holder,
                    (w as f32 / max_width as f32, h as f32 / max_height as f32),
                    (max_width, max_height),
                    &path_collection,
                );
            }
            #[cfg(target_os = "linux")]
            {
                generate_windows(&event_loop, &mut stuff_holder, &path_collection);
            }
        }
    }

    // let max_width = 2;
    // let max_height = 2;
    // for w in 0..max_width {
    //     for h in 0..max_height {
    //         generate_sub_window(&window_two, &event_loop, &mut stuff_holder, (w as f32 / max_width as f32, h as f32 / max_height as f32), (max_width, max_height));
    //     }
    // }

    // let mut counting_instant_array = [(window_one, None)];

    event_loop.run(move |event, _, control_flow| {
        let start_time = std::time::Instant::now();
        *control_flow = ControlFlow::WaitUntil(start_time + Duration::from_millis(20));
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("The close button was pressed; stopping");
                std::process::exit(0);

                // *control_flow = ControlFlow::Exit;
                // return;
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                window_id,
            } => {
                // if input.virtual_keycode == Some(VirtualKeyCode::P)
                //     && input.state == ElementState::Pressed
                // {
                //     let (window, _) = counting_instant_array
                //         .iter()
                //         .find(|c| c.0.id() == window_id)
                //         .unwrap();
                //     window.set_fullscreen(
                //         window.fullscreen().xor(Some(Fullscreen::Borderless(None))),
                //     );
                // }

                if input.virtual_keycode == Some(VirtualKeyCode::Escape) {
                    // *control_flow = ControlFlow::Exit;
                    // return;
                    std::process::exit(0);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                window_id,
            } =>
            {
                #[cfg(target_os = "windows")]
                {
                    for ele in stuff_holder
                        .iter()
                        .filter(|c| c.parent_window_id == window_id)
                    {
                        ele.child_window.set_inner_size(PhysicalSize::new(
                            new_size.width / ele.max.0,
                            new_size.height / ele.max.1,
                        ));
                        ele.child_window.set_outer_position(PhysicalPosition::new(
                            new_size.width as f32 * ele.position.0,
                            new_size.height as f32 * ele.position.1,
                        ));
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                window_id,
            } => {
                if state == ElementState::Pressed {
                    if let Some(stuff) = stuff_holder
                        .iter()
                        .find(|c| c.child_window.id() == window_id)
                    {
                        if button == MouseButton::Right {
                            println!("{:?}", stuff.mpv.get_property::<String>("filename"));
                        } else if button == MouseButton::Left {
                        }
                    }
                    println!("Right clicked");
                }
            }
            Event::MainEventsCleared => {
                // Application update code.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw, in
                // applications which do not always need to. Applications that redraw continuously
                // can just render here instead.
                // window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                // pixels.get_frame()[0] = 255;
                // pixels.render();

                // println!("\nredrawing!\n");
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in MainEventsCleared, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.
            }
            _ => (),
        }

        for ele in &mut stuff_holder {
            if let Some(ev) = ele.mpv.event_context_mut().wait_event(0.0) {
                match ev {
                    Ok(MpvEvent::StartFile) => {
                        println!("File loaded");

                        append_random_playlist(&ele.mpv, &path_collection);
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
    });
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

#[cfg(target_os = "windows")]
struct ItemHolderWindows {
    child_window: Window,
    parent_window_id: WindowId,
    mpv: Mpv,
    position: (f32, f32),
    max: (u32, u32),
}

#[cfg(target_os = "linux")]
struct ItemHolderLinux {
    child_window: Window,
    mpv: Mpv,
}

#[cfg(target_os = "linux")]
fn generate_windows(
    event_loop: &EventLoop<()>,
    child_window_holder: &mut Vec<ItemHolderLinux>,
    path_collection: &impl RandomPathGenerator,
) {
    let child_window = WindowBuilder::new().build(event_loop).unwrap();

    let mpv = mpv_setup();

    append_random_playlist(&mpv, path_collection);
    append_random_playlist(&mpv, path_collection);

    child_window_holder.push(ItemHolderLinux { child_window, mpv });
}

#[cfg(target_os = "windows")]
fn generate_sub_window(
    parent_window: &Window,
    event_loop: &EventLoop<()>,
    child_window_holder: &mut Vec<ItemHolderWindows>,
    position: (f32, f32),
    max: (u32, u32),
    path_collection: &impl RandomPathGenerator,
) {
    let innersize = parent_window.inner_size();
    let child_window = WindowBuilder::new()
        .with_decorations(false)
        .with_parent_window(parent_window.hwnd() as *mut winapi::shared::windef::HWND__)
        .with_inner_size(PhysicalSize::new(
            innersize.width / max.0,
            innersize.height / max.1,
        ))
        .with_position(PhysicalPosition::new(
            innersize.width as f32 * position.0,
            innersize.height as f32 * position.1,
        ))
        .build(event_loop)
        .unwrap();

    let mpv = mpv_setup();

    let id = {
        #[cfg(target_os = "windows")]
        {
            dbg!(child_window.hwnd()) as i64
        }
        #[cfg(target_os = "linux")]
        {
            dbg!(child_window.xlib_window()).unwrap() as i64
        }
    };
    mpv.set_property("wid", id).unwrap();

    append_random_playlist(&mpv, path_collection);
    append_random_playlist(&mpv, path_collection);

    child_window_holder.push(ItemHolderWindows {
        child_window,
        mpv,
        position,
        max,
        parent_window_id: parent_window.id(),
    });
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
