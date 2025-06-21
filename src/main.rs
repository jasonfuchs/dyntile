use wayland_client::Connection;
use wayland_client::Dispatch;
use wayland_client::delegate_noop;

use wayland_client::protocol::wl_output;
use wayland_client::protocol::wl_registry;

use river_layout::protocol::river_layout_manager_v3;
use river_layout::protocol::river_layout_v3;

fn main() {
    let conn = Connection::connect_to_env().unwrap();

    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    let display = conn.display();
    display.get_registry(&qh, ());

    let mut state = State {
        layout_manager: None,
        tags: None,
        should_exit: false,
    };

    while !state.should_exit {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }
}

struct State {
    layout_manager: Option<river_layout_manager_v3::RiverLayoutManagerV3>,
    tags: Option<u32>,
    should_exit: bool,
}

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _conn: &Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        dbg!(&event);

        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match &interface[..] {
                "wl_output" => {
                    let _output: wl_output::WlOutput = registry.bind(name, version, &qh, ());
                }
                "river_layout_manager_v3" => {
                    let layout_manager: river_layout_manager_v3::RiverLayoutManagerV3 =
                        registry.bind(name, version, &qh, ());

                    state.layout_manager = Some(layout_manager);
                }
                _ => (),
            }
        }
    }
}

impl Dispatch<wl_output::WlOutput, ()> for State {
    fn event(
        state: &mut Self,
        output: &wl_output::WlOutput,
        event: wl_output::Event,
        _: &(),
        _conn: &Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        dbg!(&event);

        match event {
            wl_output::Event::Name { .. } => {
                if let Some(ref layout_manager) = state.layout_manager {
                    let _layout: river_layout_v3::RiverLayoutV3 =
                        layout_manager.get_layout(output, "dyntile".into(), &qh, ());
                }
            }
            _ => (),
        }
    }
}

delegate_noop!(State: ignore river_layout_manager_v3::RiverLayoutManagerV3 );

impl Dispatch<river_layout_v3::RiverLayoutV3, ()> for State {
    fn event(
        state: &mut Self,
        layout: &river_layout_v3::RiverLayoutV3,
        event: river_layout_v3::Event,
        _: &(),
        _conn: &Connection,
        _qh: &wayland_client::QueueHandle<Self>,
    ) {
        dbg!(&event);

        match event {
            river_layout_v3::Event::UserCommandTags { tags } => state.tags = Some(tags),
            river_layout_v3::Event::NamespaceInUse => state.should_exit = true,
            river_layout_v3::Event::LayoutDemand {
                view_count,
                usable_width,
                usable_height,
                tags: _,
                serial,
            } => {
                for _ in 0..view_count {
                    layout.push_view_dimensions(0, 0, usable_width, usable_height, serial);
                }

                layout.commit("[ ]".into(), serial);
            }
            _ => (),
        }
    }
}
