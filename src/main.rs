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

    let mut state = State::Uninit;

    while let State::Uninit { .. } | State::Init { .. } = state {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }
}

enum State {
    Uninit,
    Init {
        layout_manager: river_layout_manager_v3::RiverLayoutManagerV3,
        tags: Option<u32>,
    },
    ShouldExit,
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

                    *state = State::Init {
                        layout_manager,
                        tags: None,
                    };
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

        match state {
            State::Init { layout_manager, .. } => match event {
                wl_output::Event::Name { .. } => {
                    let _layout: river_layout_v3::RiverLayoutV3 =
                        layout_manager.get_layout(output, "dyntile".into(), &qh, ());
                }
                _ => (),
            },
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

        match state {
            State::Init { tags, .. } => match event {
                river_layout_v3::Event::UserCommandTags {
                    tags: user_command_tags,
                } => *tags = Some(user_command_tags),
                river_layout_v3::Event::NamespaceInUse => *state = State::ShouldExit,
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
            },
            _ => (),
        }
    }
}
