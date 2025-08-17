use minimal_winit_android::_main;
use winit::event_loop::EventLoop;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info) // Default Log Level
        .parse_default_env()
        .init();
    let event_loop = EventLoop::new().unwrap();
    log::info!("Hello from desktop!");
    _main(event_loop);
}
