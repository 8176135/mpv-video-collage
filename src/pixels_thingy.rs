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
use libmpv::render::{RenderContext, RenderParam, RenderParamApiType};

#[derive(Debug)]
enum UserEvent {
    MpvEventAvailable,
    RedrawRequested,
}

pub fn running() {
    let event_loop = EventLoop::<UserEvent>::with_user_event();
    let window_one = WindowBuilder::new()
        .with_maximized(false)
        .build(&event_loop)
        .unwrap();

    let mut pixels = {
        let window_size = window_one.inner_size();
        println!("{:?}", window_size);
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window_one);
        Pixels::new(window_size.width, window_size.height, surface_texture).unwrap()
    };

    let mut mpv = Mpv::new().expect("Error while creating MPV");

    mpv.set_property("volume", 50).unwrap();

    mpv.set_property("panscan", 1).unwrap();
    // mpv.set_property("prefetch-playlist", "yes").unwrap();
    // mpv.set_property("cache", "yes").unwrap();

    let mut render_context = RenderContext::new(
        unsafe { mpv.ctx.as_mut() },
        vec![
            RenderParam::<()>::ApiType(RenderParamApiType::Software),
            // RenderParam::<()>::AdvancedControl(true),
        ],
    )
        .expect("Failed creating render context");

    mpv.event_context_mut().disable_deprecated_events().unwrap();

    let event_proxy = event_loop.create_proxy();
    render_context.set_update_callback(move || {
        event_proxy.send_event(UserEvent::RedrawRequested).unwrap();
    });
    let event_proxy = event_loop.create_proxy();
    mpv.event_context_mut().set_wakeup_callback(move || {
        event_proxy
            .send_event(UserEvent::MpvEventAvailable)
            .unwrap();
    });

    mpv.playlist_load_files(&[(&"aaa.mp4", FileState::AppendPlay, None)])
        .unwrap();

    let mut render_context = Some(render_context);

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
                    pixels.resize_buffer(new_size.width, new_size.height);
                    pixels.resize_surface(new_size.width, new_size.height);
                }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                window_id,
            } => {
                if state == ElementState::Pressed {
                    println!("Right clicked");
                }
            }
            Event::UserEvent(UserEvent::RedrawRequested) => {
                window_one.request_redraw();
            }
            Event::UserEvent(UserEvent::MpvEventAvailable) => loop {
                match mpv.event_context_mut().wait_event(0.0) {
                    Some(Ok(libmpv::events::Event::EndFile(_))) => {
                        *control_flow = ControlFlow::Exit;
                        break;
                    }
                    Some(Ok(mpv_event)) => {
                        eprintln!("MPV event: {:?}", mpv_event);
                    }
                    Some(Err(err)) => {
                        eprintln!("MPV Error: {}", err);
                        *control_flow = ControlFlow::Exit;
                        break;
                    }
                    None => {
                        *control_flow = ControlFlow::Wait;
                        break;
                    }
                }
            },
            Event::LoopDestroyed => {
                render_context.take(); // It's important to destroy the render context before the mpv player!
            }
            // Event::MainEventsCleared => {
            //     // Application update code.
            //
            //     // Queue a RedrawRequested event.
            //     //
            //     // You only need to call this if you've determined that you need to redraw, in
            //     // applications which do not always need to. Applications that redraw continuously
            //     // can just render here instead.
            //     // window.request_redraw();
            // }
            Event::RedrawRequested(_) => {

                if let Some(render_context) = &render_context {
                    let frame = pixels.get_frame();
                    
                    let window_size = window_one.inner_size();
                    let stride = (window_size.width * 4) as usize;
                    // println!("{}", stride);
                    // println!("{}", frame.len());

                    // println!("Frame: {}", frame[0]);
                    if frame.len() == (window_size.width * window_size.height * 4) as usize {
                        render_context.render_sw((window_size.width as i32, window_size.height as i32), "0bgr", stride, frame)
                            .expect("Failed to draw on glutin window");
                        pixels.render();
                    }
                }

                // println!("\nredrawing!\n");
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
