use nalgebra_glm::{Vec3, Mat4, look_at, perspective};
use minifb::{Key, Window, WindowOptions};
use std::time::Duration;
use std::f32::consts::PI;

mod framebuffer;
mod triangle;
mod vertex;
mod obj;
mod color;
mod fragment;
mod shaders;
mod camera;

use framebuffer::Framebuffer;
use vertex::Vertex;
use obj::Obj;
use camera::Camera;
use triangle::triangle;
use shaders::{vertex_shader};
use fastnoise_lite::{FastNoiseLite, NoiseType};
use crate::shaders::tatooine_shader;
use crate::shaders::kamino_shader;
use crate::shaders::sol_shader;
use crate::shaders::hoth_shader;
use crate::shaders::death_star_shader;
use crate::fragment::Fragment;
use crate::color::Color;


pub struct Uniforms {
    model_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    viewport_matrix: Mat4,
    time: u32,
    noise: FastNoiseLite
}

fn create_noise() -> FastNoiseLite {
    create_cloud_noise()
}

fn create_cloud_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise
}

fn create_model_matrix(translation: Vec3, scale: f32, rotation: Vec3) -> Mat4 {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    let rotation_matrix_x = Mat4::new(
        1.0,  0.0,    0.0,   0.0,
        0.0,  cos_x, -sin_x, 0.0,
        0.0,  sin_x,  cos_x, 0.0,
        0.0,  0.0,    0.0,   1.0,
    );

    let rotation_matrix_y = Mat4::new(
        cos_y,  0.0,  sin_y, 0.0,
        0.0,    1.0,  0.0,   0.0,
        -sin_y, 0.0,  cos_y, 0.0,
        0.0,    0.0,  0.0,   1.0,
    );

    let rotation_matrix_z = Mat4::new(
        cos_z, -sin_z, 0.0, 0.0,
        sin_z,  cos_z, 0.0, 0.0,
        0.0,    0.0,  1.0, 0.0,
        0.0,    0.0,  0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    let transform_matrix = Mat4::new(
        scale, 0.0,   0.0,   translation.x,
        0.0,   scale, 0.0,   translation.y,
        0.0,   0.0,   scale, translation.z,
        0.0,   0.0,   0.0,   1.0,
    );

    transform_matrix * rotation_matrix
}


fn create_view_matrix(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    look_at(&eye, &center, &up)
}

fn create_perspective_matrix(window_width: f32, window_height: f32) -> Mat4 {
    let fov = 45.0 * PI / 180.0;
    let aspect_ratio = window_width / window_height;
    let near = 0.1;
    let far = 1000.0;

    perspective(fov, aspect_ratio, near, far)
}

fn create_viewport_matrix(width: f32, height: f32) -> Mat4 {
    Mat4::new(
        width / 2.0, 0.0, 0.0, width / 2.0,
        0.0, -height / 2.0, 0.0, height / 2.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    )
}
fn render(
    framebuffer: &mut Framebuffer,
    uniforms: &Uniforms,
    vertex_array: &[Vertex],
    shader_fn: &dyn Fn(&Fragment, &Uniforms) -> Color,
) {
    // Vertex Shader
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

    // Primitive Assembly
    let mut triangles = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }

    // Rasterization
    let mut fragments = Vec::new();
    for tri in &triangles {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2]));
    }

    // Fragment Processing
    for fragment in fragments {
        let x = fragment.position.x as usize;
        let y = fragment.position.y as usize;

        if x < framebuffer.width && y < framebuffer.height {
            let shaded_color = shader_fn(&fragment, uniforms);
            let color = shaded_color.to_hex();
            framebuffer.set_current_color(color);
            framebuffer.point(x, y, fragment.depth);
        }
    }
}

fn calculate_orbit_position(time: f32, orbit_radius: f32, angular_velocity: f32) -> Vec3 {
    let x = orbit_radius * (time * angular_velocity).cos();
    let z = orbit_radius * (time * angular_velocity).sin();
    Vec3::new(x, 0.0, z)
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Proyecto 3",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    window.set_position(500, 500);

    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 10.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let obj = Obj::load("assets/models/sphere-1.obj").expect("Failed to load obj");
    let vertex_arrays = obj.get_vertex_array();
    let mut time = 0;

    let solar_objects: Vec<(Box<dyn Fn(&Fragment, &Uniforms) -> Color>, Vec3, f32, f32)> = vec![
        (Box::new(sol_shader), Vec3::new(0.0, 0.0, 0.0), 1.5, 0.0),  
        (Box::new(tatooine_shader), Vec3::new(3.0, 0.0, 0.0), 0.5, 0.01),  
        (Box::new(hoth_shader), Vec3::new(5.0, 0.0, 0.0), 0.4, 0.012),   
        (Box::new(kamino_shader), Vec3::new(0.0, 6.0, 0.0), 0.6, 0.014), 
        (Box::new(death_star_shader), Vec3::new(0.0, -4.0, 0.0), 0.7, 0.016), 
    ];

    let mut current_planet_index = 0; 

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.is_key_pressed(Key::N, minifb::KeyRepeat::No) {
            current_planet_index = (current_planet_index + 1) % solar_objects.len(); 
            camera.move_to_next_planet(&solar_objects, current_planet_index);
        }
    
        handle_input(&window, &mut camera);
        framebuffer.clear();
        framebuffer.set_background_color(0x000000); 

        (&mut framebuffer).draw_stars(15); 
        time += 1;
        
    
        let view_matrix = create_view_matrix(camera.eye, camera.center, camera.up);
        let projection_matrix = create_perspective_matrix(window_width as f32, window_height as f32);
        let viewport_matrix = create_viewport_matrix(framebuffer_width as f32, framebuffer_height as f32);
    
        for (shader_fn, initial_translation, scale, orbital_speed) in &solar_objects {
            let angle = time as f32 * orbital_speed;  
            let translation = Vec3::new(
                initial_translation.x * angle.cos() - initial_translation.y * angle.sin(),
                initial_translation.x * angle.sin() + initial_translation.y * angle.cos(),
                initial_translation.z,
            );
        
            let rotation = Vec3::new(0.0, time as f32 * 0.01, 0.0);  
            let model_matrix = create_model_matrix(translation, *scale, rotation);
        
            let uniforms = Uniforms { 
                model_matrix, 
                view_matrix: view_matrix.clone(), 
                projection_matrix: projection_matrix.clone(), 
                viewport_matrix: viewport_matrix.clone(),
                time,
                noise: create_noise(),
            };
        
            render(&mut framebuffer, &uniforms, &vertex_arrays, shader_fn);
        }
        
    
        window.update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height).unwrap();
        std::thread::sleep(frame_delay);
    }
}



fn handle_input(window: &Window, camera: &mut Camera) {
    let movement_speed = 1.0;
    let rotation_speed = PI/50.0;
    let zoom_speed = 0.1;
   
    //  camera orbit controls
    if window.is_key_down(Key::Left) {
      camera.orbit(rotation_speed, 0.0);
    }
    if window.is_key_down(Key::Right) {
      camera.orbit(-rotation_speed, 0.0);
    }
    if window.is_key_down(Key::W) {
      camera.orbit(0.0, -rotation_speed);
    }
    if window.is_key_down(Key::S) {
      camera.orbit(0.0, rotation_speed);
    }

    // Camera movement controls
    let mut movement = Vec3::new(0.0, 0.0, 0.0);
    if window.is_key_down(Key::A) {
      movement.x -= movement_speed;
    }
    if window.is_key_down(Key::D) {
      movement.x += movement_speed;
    }
    if window.is_key_down(Key::Q) {
      movement.y += movement_speed;
    }
    if window.is_key_down(Key::E) {
      movement.y -= movement_speed;
    }
    if movement.magnitude() > 0.0 {
      camera.move_center(movement);
    }

    // Camera zoom controls
    if window.is_key_down(Key::Up) {
      camera.zoom(zoom_speed);
    }
    if window.is_key_down(Key::Down) {
      camera.zoom(-zoom_speed);
    }
}