use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    env_logger::init();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut gpu = runtime.block_on(async {
        tokio::spawn(async move {
            loop {
                let event = event_rx.recv().await.unwrap();

                println!("{:?}", event);
            }
        });

       gpu::Gpu::new(&window).await
    });

    event_loop.run(move |event, _target, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                gpu.update();

                match gpu.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => gpu.resize(gpu.size()),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !gpu.input(event) {
                    match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                        }
                        WindowEvent::Resized(physical_size) => {
                            gpu.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            gpu.resize(**new_inner_size)
                        }
                        _ => {}
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }

        if let Some(event) = event.to_static() {
            event_tx.send(event).unwrap();
        }
    });
}

mod gpu;
