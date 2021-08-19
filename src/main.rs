use pixels::Pixels;
use pixels::SurfaceTexture;
use winit::dpi::PhysicalPosition;
use winit::event::ElementState;
use winit::event::MouseButton;
use winit::event::VirtualKeyCode;

use libmpv::{
    events::*,
    render::{OpenGLInitParams, RenderContext, RenderFrameInfo, RenderParam},
    *,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::windows::WindowBuilderExtWindows,
    platform::windows::WindowExtWindows,
    window::WindowBuilder,
    dpi::PhysicalSize,
    window::Window,
};
// use std::{collections::HashMap, env, ffi::c_void, thread, time::Duration};

// const VIDEO_URL: &str = "https://www.youtube.com/watch?v=DLzxrzFCyOs";

// fn main() -> Result<()> {

//     // use glium::{glutin, Surface};

//     // let event_loop = glutin::event_loop::EventLoop::new();
//     // let wb = glutin::window::WindowBuilder::new();
//     // let cb = glutin::ContextBuilder::new();
//     // let display = glium::Display::new(wb, cb, &event_loop).unwrap();

//     // event_loop.run(move |event, _, control_flow| {
//     //     let next_frame_time = std::time::Instant::now() +
//     //         std::time::Duration::from_nanos(16_666_667);
//     //     *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

//     //     match event {
//     //         glutin::event::Event::WindowEvent { event, .. } => match event {
//     //             glutin::event::WindowEvent::CloseRequested => {
//     //                 *control_flow = glutin::event_loop::ControlFlow::Exit;
//     //                 return;
//     //             },
//     //             _ => return,
//     //         },
//     //         glutin::event::Event::NewEvents(cause) => match cause {
//     //             glutin::event::StartCause::ResumeTimeReached { .. } => (),
//     //             glutin::event::StartCause::Init => (),
//     //             _ => return,
//     //         },
//     //         _ => return,
//     //     }

//     //     let mut target = display.draw();
//     //     target.clear_color(0.0, 0.0, 1.0, 1.0);
//     //     target.finish().unwrap();
//     // });

//     // Create an `Mpv` and set some properties.
//     let mut mpv = Mpv::new()?;
//     // mpv.set_property("volume", 15)?;
//     // mpv.set_property("vo", "null")?;

//     let mut ev_ctx = mpv.event_context();
//     ev_ctx.disable_deprecated_events()?;
//     ev_ctx.observe_property("volume", Format::Int64, 0)?;
//     ev_ctx.observe_property("demuxer-cache-state", Format::Node, 0)?;

//     mpv.playlist_load_files(&[(&path, FileState::AppendPlay, None)])
//         .unwrap();
//     mpv.set_property(name, data)
//     mpv.set_property("volume", 100).unwrap();
//     // RenderParam::NextFrameInfo(RenderFrameInfo {flags: render::RenderFrameInfoFlag::Present,})
//     let context = libmpv::render::RenderContext::new(unsafe { mpv.ctx.as_mut() }, vec![RenderParam::InitParams(OpenGLInitParams {get_proc_address: something_else, ctx: ()}),]).unwrap();
//     context.render::<()>(3, 1920, 1080, false).unwrap();
//     loop {
//         let ev = mpv
//             .event_context_mut()
//             .wait_event(600.)
//             .unwrap_or(Err(Error::Null));

//         match ev {
//             Ok(Event::EndFile(r)) => {
//                 println!("Exiting! Reason: {:?}", r);
//                 break;
//             }

//             Ok(Event::PropertyChange {
//                 name: "demuxer-cache-state",
//                 change: PropertyData::Node(mpv_node),
//                 ..
//             }) => {
//                 let ranges = seekable_ranges(mpv_node).unwrap();
//                 println!("Seekable ranges updated: {:?}", ranges);
//             }
//             Ok(e) => println!("Event triggered: {:?}", e),
//             Err(e) => println!("Event errored: {:?}", e),
//         }
//     }
//     Ok(())
// }

// fn something_else(ctx: &(), other: &str) -> *mut c_void {
//     dbg!(other);
//     std::ptr::null_mut()
// }

// fn seekable_ranges(demuxer_cache_state: &MpvNode) -> Option<Vec<(f64, f64)>> {
//     let mut res = Vec::new();
//     let props: HashMap<&str, MpvNode> = demuxer_cache_state.to_map()?.collect();
//     let ranges = props.get("seekable-ranges")?.to_array()?;

//     for node in ranges {
//         let range: HashMap<&str, MpvNode> = node.to_map()?.collect();
//         let start = range.get("start")?.to_f64()?;
//         let end = range.get("end")?.to_f64()?;
//         res.push((start, end));
//     }

//     Some(res)
// }

const WIDTH: u32 = 2560;
const HEIGHT: u32 = 1440;

fn main() {

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_inner_size(PhysicalSize::new(2000, 1000)).build(&event_loop).unwrap();
    // let window_handle = window.hwnd() as usize;
    let other_window = WindowBuilder::new()
        .with_decorations(false)
        .with_parent_window(window.hwnd() as *mut winapi::shared::windef::HWND__)
        .with_inner_size(PhysicalSize::new(WIDTH / 2, HEIGHT / 2))
        .build(&event_loop)
        .unwrap();

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(100, 100, surface_texture).unwrap()
    };

    let mut stuff_holder = Vec::new();
    generate_sub_window(&window, &event_loop, &mut stuff_holder, (0, 0));
    // generate_sub_window(&window, &event_loop, &mut stuff_holder, (WIDTH / 2, 0));

    let path = std::env::args().nth(1).unwrap();

    event_loop.run(move |event, _, control_flow| {
        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
        // dispatched any events. This is ideal for games and similar applications.
        // *control_flow = ControlFlow::Poll;

        // ControlFlow::Wait pauses the event loop if no events are available to process.
        // This is ideal for non-game applications that only update in response to user
        // input, and uses significantly less power/CPU time than ControlFlow::Poll.
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("The close button was pressed; stopping");
                *control_flow = ControlFlow::Exit
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                window_id,
            } => {
                if input.virtual_keycode == Some(VirtualKeyCode::P)
                    && input.state == ElementState::Pressed
                {
                    println!("P pressed");
                    for ele in &stuff_holder {
                        ele.mpv.playlist_load_files(&[(&path, FileState::AppendPlay, None)])
                        .unwrap();
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                window_id,
            } => {
                if button == MouseButton::Right && state == ElementState::Pressed {
                    if window_id != window.id() {
                        println!("Wrong window");
                        return;
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
                pixels.get_frame()[0] = 255;
                pixels.render().unwrap();

                println!("\nredrawing!\n");
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in MainEventsCleared, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.
            }
            _ => (),
        }
    });
}


struct ItemHolder {
    child_window: Window,
    mpv: Mpv,
}

fn generate_sub_window(parent_window: &Window, event_loop: &EventLoop<()>, child_window_holder: &mut Vec<ItemHolder> ,position: (u32, u32)) {

    let child_window = WindowBuilder::new()
        .with_decorations(false)
        .with_parent_window(parent_window.hwnd() as *mut winapi::shared::windef::HWND__)
        .with_inner_size(PhysicalSize::new(WIDTH / 2, HEIGHT / 2))
        // .with_position(PhysicalPosition::new(position.0, position.1))
        .build(event_loop)
        .unwrap();
    
    let mpv = Mpv::new().unwrap();
                    mpv.set_property("volume", 100).unwrap();
                    mpv.set_property("wid", child_window.hwnd() as i64).unwrap();

    let ev_ctx = mpv.event_context();

    ev_ctx.disable_deprecated_events().unwrap();
    ev_ctx.observe_property("volume", Format::Int64, 0).unwrap();
    ev_ctx.observe_property("demuxer-cache-state", Format::Node, 0).unwrap();
    println!("Mpv generated");
    child_window_holder.push(ItemHolder {
        child_window,
        mpv,
    });


}