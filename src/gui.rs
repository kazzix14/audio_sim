use crate::SIZE;

use std::rc::Rc;
use std::sync::*;
use std::time::*;

use glium::backend::Context;
use glium::backend::Facade;
use glium::index::PrimitiveType;
use glium::texture::*;
use glium::Texture2d;

use imgui::*;
use imgui_glium_renderer::*;
use imgui_winit_support::*;

pub type Textures = imgui::Textures<Texture2d>;

pub fn run<F>(title: String, mem: Arc<Mutex<Vec<f32>>>, mut run_ui: F)
where
    F: FnMut(&mut bool, &mut Ui) -> bool,
{
    use glium::glutin;
    use glium::{Display, Surface};
    use imgui_glium_renderer::Renderer;

    let mut events_loop = glutin::EventsLoop::new();

    let wb = glutin::WindowBuilder::new().with_title(title);

    let cb = glutin::ContextBuilder::new().with_vsync(true);

    let display = Display::new(wb, cb, &events_loop).unwrap();

    let mic_image = image::load(
        std::io::Cursor::new(&include_bytes!("../resources/images/mic.png")[..]),
        image::PNG,
    )
    .unwrap()
    .to_rgba();

    let mic_image_dimensions = mic_image.dimensions();

    let mic_image = glium::texture::RawImage2d::from_raw_rgba_reversed(
        &mic_image.into_raw(),
        mic_image_dimensions,
    );

    //let mic_opengl_texture = glium::texture::

    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);

    let gl_window = display.gl_window();
    let window = gl_window.window();
    let mut platform = WinitPlatform::init(&mut imgui);
    platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Rounded);

    let hidpi_factor = platform.hidpi_factor();
    let font_size = (13.0 * hidpi_factor) as f32;

    imgui.fonts().add_font(&[FontSource::DefaultFontData {
        config: Some(FontConfig {
            size_pixels: font_size,
            ..FontConfig::default()
        }),
    }]);

    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

    let mut renderer = Renderer::init(&mut imgui, &display).unwrap();

    //imgui_winit_support::configure_keys(&mut imgui);

    let mut last_frame = Instant::now();
    let mut run = true;

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

    while run {
        events_loop.poll_events(|event| {
            use glium::glutin::{Event, WindowEvent::CloseRequested};

            platform.handle_event(imgui.io_mut(), &window, &event);

            if let Event::WindowEvent { event, .. } = event {
                match event {
                    CloseRequested => run = false,
                    _ => (),
                }
            }
        });

        let io = imgui.io_mut();

        let now = Instant::now();

        platform.prepare_frame(io, &window).unwrap();
        last_frame = io.update_delta_time(last_frame);
        let mut ui = imgui.frame();

        run_ui(&mut run, &mut ui);
        //imgui_winit_support::update_mouse_cursor(&imgui, &window);

        //if !run_ui(&ui, display.get_context(), renderer.textures()) {
        //quit = true;
        //}

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

        platform.prepare_render(&ui, &window);
        let draw_data = ui.render();
        renderer.render(&mut target, draw_data).unwrap();
        target.finish().unwrap();
    }
}
