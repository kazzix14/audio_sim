use crate::SIZE;

use std::rc::Rc;
use std::sync::*;
use std::time::*;

use glium::backend::Context;
use glium::backend::Facade;
use glium::Texture2d;
use imgui::*;
use imgui_glium_renderer::*;
use imgui_winit_support::*;

pub type Textures = imgui::Textures<Texture2d>;

pub fn run<F>(title: String, mem: Arc<Mutex<Vec<f32>>>, mut run_ui: F)
where
    F: FnMut(&Ui, &Rc<Context>, &mut Textures) -> bool,
{
    use glium::glutin;
    use glium::{Display, Surface};
    use imgui_glium_renderer::Renderer;

    let mut events_loop = glutin::EventsLoop::new();

    let wb = glutin::WindowBuilder::new().with_title(title);

    let cb = glutin::ContextBuilder::new().with_vsync(true);

    let display = Display::new(wb, cb, &events_loop).unwrap();

    let window = display.gl_window();

    let mut imgui = ImGui::init();

    imgui.set_ini_filename(None);

    let hidpi_factor = window.get_hidpi_factor().round();

    let font_size = (13.0 * hidpi_factor) as f32;

    imgui.fonts().add_default_font_with_config(
        ImFontConfig::new()
            .oversample_h(1)
            .pixel_snap_h(true)
            .size_pixels(font_size),
    );

    let mut renderer = Renderer::init(&mut imgui, &display).unwrap();

    imgui_winit_support::configure_keys(&mut imgui);

    let mut last_frame = Instant::now();
    let mut quit = false;

    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
    }

    implement_vertex!(Vertex, position);

    let v1 = Vertex {
        position: [-1.0, -1.0],
    };
    let v2 = Vertex {
        position: [1.0, 1.0],
    };
    let v3 = Vertex {
        position: [-1.0, 1.0],
    };
    let v4 = Vertex {
        position: [-1.0, -1.0],
    };
    let v5 = Vertex {
        position: [1.0, 1.0],
    };
    let v6 = Vertex {
        position: [1.0, -1.0],
    };
    let shape = vec![v1, v2, v3, v4, v5, v6];
    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let vertex_shader_src = format!(
        r#"
        #version 430

        in vec2 position;

        void main()
        {{
            gl_Position = vec4(position, 0.0, 1.0);
            
        }}
    "#
    );

    let fragment_shader_src = format!(
        r#"
        #version 430

        out vec4 color;
        layout(std140) buffer MyBlock
        {{
            vec4 values[{len}];
        }};
        uniform float width;
        uniform float height;

        void main()
        {{
            uint x = uint((gl_FragCoord.x / width) * float({size}));
            uint y = uint((gl_FragCoord.y / height) * float({size}));

            uint i = (x+y*{size})/4;
            uint j = (x+y*{size})%4;

            vec4 value_vec = values[i];

            float value = 0;
            switch (j)
            {{
                case 0:
                    value = value_vec.x;
                    break;
                case 1:
                    value = value_vec.y;
                    break;
                case 2:
                    value = value_vec.z;
                    break;
                case 3:
                    value = value_vec.w;
                    break;
            }}

            color = vec4(value/20.0, value/5.0, value, 1.0);
        }}
    "#,
        len = SIZE * SIZE,
        size = SIZE
    );

    let program = glium::Program::from_source(
        &display,
        vertex_shader_src.as_str(),
        fragment_shader_src.as_str(),
        None,
    )
    .unwrap();

    struct Data {
        values: [[f32; 4]],
    }

    implement_buffer_content!(Data);
    implement_uniform_block!(Data, values);

    let mut buffer: glium::uniforms::UniformBuffer<Data> =
        glium::uniforms::UniformBuffer::empty_unsized(&display, 4 * SIZE * SIZE).unwrap();

    while !quit {
        events_loop.poll_events(|event| {
            use glium::glutin::{Event, WindowEvent::CloseRequested};

            imgui_winit_support::handle_event(
                &mut imgui,
                &event,
                window.get_hidpi_factor(),
                hidpi_factor,
            );

            if let Event::WindowEvent { event, .. } = event {
                match event {
                    CloseRequested => quit = true,
                    _ => (),
                }
            }
        });

        let now = Instant::now();
        let delta = now - last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        last_frame = now;

        imgui_winit_support::update_mouse_cursor(&imgui, &window);

        let frame_size = imgui_winit_support::get_frame_size(&window, hidpi_factor).unwrap();

        let ui = imgui.frame(frame_size, delta_s);
        if !run_ui(&ui, display.get_context(), renderer.textures()) {
            quit = true;
        }

        let logical_size = window.get_inner_size().unwrap();
        let mut target = display.draw();
        target.clear_color(0.0, 1.0, 1.0, 1.0);

        {
            let vec: Vec<f32>;
            {
                let m = mem.lock().unwrap();
                vec = m.clone();
            }

            let mut mapping = buffer.map();
            for (i, val) in mapping.values.iter_mut().enumerate() {
                *val = [vec[i * 4], vec[i * 4 + 1], vec[i * 4 + 2], vec[i * 4 + 3]];
            }
        }

        target.draw(&vertex_buffer, &indices, &program, &uniform!{MyBlock: &*buffer, width: logical_size.width as f32, height: logical_size.height as f32}, &Default::default()).unwrap();

        renderer.render(&mut target, ui).unwrap();
        target.finish().unwrap();
    }
}
