
use nalgebra_glm::{Vec3, Vec4, Mat3, dot, mat4_to_mat3};
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::fragment::Fragment;
use crate::color::Color;
use std::f32::consts::PI;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    let position = Vec4::new(
        vertex.position.x,
        vertex.position.y,
        vertex.position.z,
        1.0
    );

    let transformed = uniforms.projection_matrix * uniforms.view_matrix * uniforms.model_matrix * position;

    let w = transformed.w;
    let transformed_position = Vec4::new(
        transformed.x / w,
        transformed.y / w,
        transformed.z / w,
        1.0
    );

    let screen_position = uniforms.viewport_matrix * transformed_position;

    let model_mat3 = mat4_to_mat3(&uniforms.model_matrix);
    let normal_matrix = model_mat3.transpose().try_inverse().unwrap_or(Mat3::identity());

    let transformed_normal = normal_matrix * vertex.normal;

    Vertex {
        position: vertex.position,
        normal: vertex.normal,
        tex_coords: vertex.tex_coords,
        color: vertex.color,
        transformed_position: Vec3::new(screen_position.x, screen_position.y, screen_position.z),
        transformed_normal: transformed_normal
    }
}

pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms, current_shader: u8) -> Color {
  match current_shader {
      0 => tatooine_shader(fragment, uniforms),
      1 => death_star_shader(fragment, uniforms),
      2 => gaseoso_shader(fragment, uniforms),
      3 => kamino_shader(fragment, uniforms),
      4 => sol_shader(fragment, uniforms),
      5 => hoth_shader(fragment, uniforms),
      6 => kashyyyk_shader(fragment, uniforms),
      _ => Color::black(), 
  }
}

pub fn kamino_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 1000.0;  
    let ox = 100.0;    
    let oy = 100.0;
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;
    let t = uniforms.time as f32 * 0.8;

    let noise_value = uniforms.noise.get_noise_2d(x * zoom + ox + t, y * zoom + oy);
  
    let detail_noise_value = uniforms.noise.get_noise_2d(x * zoom * 2.0 + ox + t, y * zoom * 2.0 + oy);
    let storm_intensity = (detail_noise_value * 0.5) + 0.5;  

    let lightning = (uniforms.time as f32).sin() * 10.0;  
    let mut cloud_color = Color::new(144, 144, 144) * 0.5;  
    if storm_intensity > 0.7 && lightning > 0.9 {
        cloud_color = cloud_color * 2.0;  
    }

    let sky_color = Color::new(0, 61, 102);  
    let stormy_sky_color = sky_color * (1.0 - storm_intensity * 0.5); 

    let cloud_threshold = 0.3;
    let noise_color = if noise_value > cloud_threshold {
        cloud_color  
    } else {
        stormy_sky_color  
    };

    noise_color * fragment.intensity
}
pub fn sol_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let bright_color = Color::new(255, 255, 204); 
  let dark_color = Color::new(255, 51, 0);    

  let position = Vec3::new(
      fragment.vertex_position.x,
      fragment.vertex_position.y,
      fragment.depth
  );

  let base_frequency = 0.2;
  let pulsate_amplitude = 0.5;
  let t = uniforms.time as f32 * 0.01;

  let pulsate = (t * base_frequency).sin() * pulsate_amplitude;

  let zoom = 1000.0; 
  let noise_value1 = uniforms.noise.get_noise_3d(
      position.x * zoom,
      position.y * zoom,
      (position.z + pulsate) * zoom
  );
  let noise_value2 = uniforms.noise.get_noise_3d(
      (position.x + 1000.0) * zoom,
      (position.y + 1000.0) * zoom,
      (position.z + 1000.0 + pulsate) * zoom
  );
  let noise_value = (noise_value1 + noise_value2) * 0.5;  

  let base_color = dark_color.lerp(&bright_color, noise_value);

  let distance_from_center = position.x.hypot(position.y);  
  let radius = 0.5;  
  let falloff = (1.0 - (distance_from_center / radius).clamp(0.0, 1.0)).powf(2.0);  

  let brightened_color = base_color * (1.0 + falloff * 2.0);  

  brightened_color * fragment.intensity
}

pub fn hoth_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let snow_color = Color::new(255, 255, 255); 
  let ice_color = Color::new(173, 216, 230);  

  let position = Vec3::new(
      fragment.vertex_position.x,
      fragment.vertex_position.y,
      fragment.depth
  );

  let zoom = 500.0;
  let t = uniforms.time as f32 * 0.01;  

  let noise_value = uniforms.noise.get_noise_3d(
      position.x * zoom,
      position.y * zoom,
      position.z * zoom + t
  );

  let ice_threshold = 0.3; 

  let base_color = if noise_value > ice_threshold {
      ice_color  
  } else {
      snow_color 
  };

  let intensity_variation = 0.9 + (noise_value * 0.1);  

  base_color * fragment.intensity * intensity_variation
}
pub fn kashyyyk_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let light_green = Color::new(144, 238, 144); 
  let medium_green = Color::new(34, 139, 34);   
  let dark_green = Color::new(139, 69, 19);      
  let terrain_color = Color::new(0, 100, 0);  

  let position = Vec3::new(
      fragment.vertex_position.x,
      fragment.vertex_position.y,
      fragment.depth
  );

  let zoom = 300.0;
  let t = uniforms.time as f32 * 0.01; 

  let noise_value = uniforms.noise.get_noise_3d(
      position.x * zoom,
      position.y * zoom,
      position.z * zoom + t
  );

  let vegetation_threshold = 0.3;  

  let vegetation_color = if noise_value > vegetation_threshold {
      if noise_value > 0.7 {
          dark_green.lerp(&medium_green, (noise_value - 0.7) * 3.0)  
      } else if noise_value > 0.5 {
          medium_green.lerp(&light_green, (noise_value - 0.5) * 2.0)  
      } else {
          light_green  
      }
  } else {
      terrain_color  
  };

  let intensity_variation = 0.9 + (noise_value * 0.1);  

  vegetation_color * fragment.intensity * intensity_variation 
}

pub fn gaseoso_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let zoom = 1000.0; 
  let ox = 50.0;    
  let oy = 50.0;    

  let x = fragment.vertex_position.x;
  let y = fragment.vertex_position.y;
  let t = uniforms.time as f32 * 0.1;

  let base_color = Color::new(128, 0, 0);        
  let band_color = Color::new(255, 204, 153);       
  let storm_color = Color::new(192, 57, 43);        
  let background_color = Color::new(0, 61, 102);    
  let noise_value = uniforms.noise.get_noise_2d(x * zoom + ox, y * zoom * 0.5 + oy + t);
  let band_intensity = (noise_value * 0.5) + 0.5;

  let storm_noise = uniforms.noise.get_noise_2d(x * zoom * 1.5 + ox, y * zoom * 1.5 + oy + t);
  let storm_intensity = (storm_noise * 0.5) + 0.5;

  let color = if band_intensity > 0.6 {
      base_color.lerp(&band_color, band_intensity)
  } else if storm_intensity > 0.7 {
      storm_color 
  } else {
      base_color 
  };

  color * fragment.intensity
}

pub fn death_star_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let position = fragment.vertex_position;
  let x = position.x;
  let y = position.y;

  let line_spacing = 0.1;
  let line_width = 0.02;  
  let circle_radius = 0.16;
  let center = Vec3::new(0.0, 0.17, 0.0); 

  let line_color = Color::new(128, 128, 128);   
  let circle_color = Color::new(64, 64, 64); 
  let background_color = Color::new(102, 102, 102); 

  let in_vertical_line = ((x / line_spacing).fract().abs() < line_width);
  let in_horizontal_line = ((y / line_spacing).fract().abs() < line_width);

  let distance_from_center = ((x - center.x).powi(2) + (y - center.y).powi(2)).sqrt();
  let in_circle = distance_from_center <= circle_radius;

  let final_color = if in_circle {
      circle_color 
  } else if in_vertical_line || in_horizontal_line {
      line_color 
  } else {
      background_color
  };

  final_color * fragment.intensity
}

pub fn tatooine_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let zoom = 1000.0;
  let time_factor = uniforms.time as f32 * 0.01; 
  let x = fragment.vertex_position.x;
  let y = fragment.vertex_position.y;

  let base_rock_color = Color::new(139, 69, 19);  
  let mountain_color = Color::new(105, 105, 105); 
  let plain_color = Color::new(205, 133, 63);     
  let land_color = Color::new(163, 163, 117);     

  let base_noise = uniforms.noise.get_noise_2d(
      x * zoom * 0.5 + time_factor,
      y * zoom * 0.5 + time_factor
  );

  let mountain_noise = uniforms.noise.get_noise_2d(
      x * zoom + time_factor * 0.5,
      y * zoom + time_factor * 0.5
  );

  let continent_shift = (uniforms.time as f32 * 0.005).sin() * 0.1;

  let continental_noise = uniforms.noise.get_noise_2d(
      (x + continent_shift) * zoom * 0.8,
      (y + continent_shift) * zoom * 0.8
  );

  let mountain_threshold = 0.6;
  let land_threshold = -0.3;

  let final_color = if base_noise > mountain_threshold {
      mountain_color.lerp(&base_rock_color, mountain_noise)
  } else if continental_noise < land_threshold {
      land_color 
  } else {
      plain_color.lerp(&base_rock_color, continental_noise) 
  };

  final_color * fragment.intensity
}

  
