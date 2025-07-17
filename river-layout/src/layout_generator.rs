use crate::prelude::*;

use crate::protocol::river_layout_manager_v3;
use crate::protocol::river_layout_v3;

use wayland_client::Connection;
use wayland_client::Dispatch;
use wayland_client::QueueHandle;
use wayland_client::delegate_noop;

use wayland_client::protocol::wl_output;
use wayland_client::protocol::wl_registry;

pub trait LayoutGenerator: 'static {
    const NAMESPACE: &'static str;
    fn run(self) -> Result<(), Error>
    where
        Self: Sized,
    {
        let conn = Connection::connect_to_env()?;

        let mut event_queue = conn.new_event_queue();
        let qh = event_queue.handle();

        let display = conn.display();
        display.get_registry(&qh, ());

        let mut state = State {
            _layout_manager: None,
            _layout_generator: self,
            _tags: None,
            _should_exit: false,
        };

        while !state._should_exit {
            event_queue.blocking_dispatch(&mut state)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct State<LG: LayoutGenerator> {
    _layout_manager: Option<river_layout_manager_v3::RiverLayoutManagerV3>,
    _layout_generator: LG,
    _tags: Option<u32>,
    _should_exit: bool,
}

impl<LG: LayoutGenerator> Dispatch<wl_registry::WlRegistry, ()> for State<LG> {
    fn event(
        _state: &mut Self,
        _registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        dbg!(&event);
    }
}

impl<LG: LayoutGenerator> Dispatch<wl_output::WlOutput, ()> for State<LG> {
    fn event(
        _state: &mut Self,
        _output: &wl_output::WlOutput,
        event: wl_output::Event,
        _: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        dbg!(&event);
    }
}

delegate_noop!(@<LG: LayoutGenerator> State<LG>: ignore river_layout_manager_v3::RiverLayoutManagerV3 );

impl<LG: LayoutGenerator> Dispatch<river_layout_v3::RiverLayoutV3, String> for State<LG> {
    fn event(
        _state: &mut Self,
        _layout: &river_layout_v3::RiverLayoutV3,
        event: river_layout_v3::Event,
        _output_name: &String,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        dbg!(&event);
    }
}
