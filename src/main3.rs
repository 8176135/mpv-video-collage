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
    platform::windows::WindowBuilderExtWindows,
    platform::windows::WindowExtWindows,
    window::Window,
    window::WindowBuilder,
};

const IGNORED_ITEMS: &[&str] = include!("../included_consts/ignored_items.rs");
const WALK_DIR_PATH: &str = include_str!("../included_consts/look_path.txt");

fn main() {
    spawn_mpv_window();
}

fn spawn_mpv_window() {
    let path_collection = walkdir::WalkDir::new(WALK_DIR_PATH)
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
        .collect::<Vec<String>>();

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

    let max_width = 3;
    let max_height = 3;
    for w in 0..max_width {
        for h in 0..max_height {
            generate_sub_window(
                &window_one,
                &event_loop,
                &mut stuff_holder,
                (w as f32 / max_width as f32, h as f32 / max_height as f32),
                (max_width, max_height),
            );
        }
    }

    // let max_width = 2;
    // let max_height = 2;
    // for w in 0..max_width {
    //     for h in 0..max_height {
    //         generate_sub_window(&window_two, &event_loop, &mut stuff_holder, (w as f32 / max_width as f32, h as f32 / max_height as f32), (max_width, max_height));
    //     }
    // }

    let mut counting_instant_array = [(window_one, None)];

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
                if input.virtual_keycode == Some(VirtualKeyCode::P)
                    && input.state == ElementState::Pressed
                {
                    let (window, _) = counting_instant_array
                        .iter()
                        .find(|c| c.0.id() == window_id)
                        .unwrap();
                    window.set_fullscreen(
                        window.fullscreen().xor(Some(Fullscreen::Borderless(None))),
                    );
                }

                if input.virtual_keycode == Some(VirtualKeyCode::Escape) {
					// *control_flow = ControlFlow::Exit;
					// return;
                    std::process::exit(0);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                window_id,
            } => {
                for (window, counting) in &mut counting_instant_array {
                    if window_id == window.id() {
                        *counting = Some((start_time, new_size));
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

        for (window, counting) in &mut counting_instant_array {
            if let Some((last_point, new_size)) = counting {
                let dur = start_time.duration_since(*last_point);
                if dur > Duration::from_millis(100) {
                    println!("Resizing");
                    for ele in stuff_holder
                        .iter()
                        .filter(|c| c.parent_window_id == window.id())
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

                    *counting = None;
                }
            }
        }

        for ele in &mut stuff_holder {
            if ele.kill_time < start_time {
                ele.mpv
                    .set_property(
                        "start",
                        format!(
                            "{}%",
                            nanorand::tls_rng().generate_range(50..950) as f64 / 10.0
                        ),
                    )
                    .ok();
                ele.mpv.playlist_next_weak().ok();
                ele.kill_time =
                    start_time + Duration::from_secs(nanorand::tls_rng().generate_range(3..30));
                ele.mpv
                    .playlist_load_files(&[(
                        &path_collection
                            [nanorand::tls_rng().generate_range(0..path_collection.len())],
                        FileState::AppendPlay,
                        None,
                    )])
                    .unwrap();
            }
        }
    });
}

struct ItemHolder {
    child_window: Window,
    parent_window_id: WindowId,
    mpv: Mpv,
    position: (f32, f32),
    max: (u32, u32),
    kill_time: Instant,
}

fn generate_sub_window(
    parent_window: &Window,
    event_loop: &EventLoop<()>,
    child_window_holder: &mut Vec<ItemHolder>,
    position: (f32, f32),
    max: (u32, u32),
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

    let mpv = Mpv::new().unwrap();
    mpv.set_property("volume", 50).unwrap();
    mpv.set_property("wid", child_window.hwnd() as i64).unwrap();
    mpv.set_property("panscan", 1).unwrap();

    let ev_ctx = mpv.event_context();

    ev_ctx.disable_deprecated_events().unwrap();
    ev_ctx.observe_property("volume", Format::Int64, 0).unwrap();
    ev_ctx
        .observe_property("demuxer-cache-state", Format::Node, 0)
        .unwrap();
    println!("Mpv generated");
    child_window_holder.push(ItemHolder {
        child_window,
        mpv,
        position,
        kill_time: Instant::now(),
        max,
        parent_window_id: parent_window.id(),
    });
}
