use wgpu::{util::DeviceExt, Device};

use crate::particle::Particle;

#[derive(Debug, Clone)]
pub struct Trail {
    time_to_live: std::time::Duration,
    position: cgmath::Point2<f64>,
    radius: f64,
    charge: f64,
}

impl Trail {
    pub fn new(
        time_to_live: std::time::Duration,
        position: cgmath::Point2<f64>,
        radius: f64,
        charge: f64,
    ) -> Self {
        Self {
            time_to_live,
            position,
            radius,
            charge,
        }
    }

    fn to_raw(&self) -> RawTrail {
        RawTrail {
            time_to_live: self.time_to_live.as_secs_f32(),
            position: [self.position.x as _, self.position.y as _],
            radius: self.radius as _,
            charge: self.charge as _,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RawTrail {
    position: [f32; 2],
    time_to_live: f32,
    radius: f32,
    charge: f32,
}

unsafe impl bytemuck::Zeroable for RawTrail {}
unsafe impl bytemuck::Pod for RawTrail {}

pub struct TrailManager {
    trails: Vec<Trail>,
}

impl TrailManager {
    pub fn new() -> Self {
        Self { trails: vec![] }
    }

    pub fn update(&mut self, dt: std::time::Duration, particles: &Vec<Particle>) {
        for trail in &mut self.trails {
            trail.time_to_live = trail.time_to_live.saturating_sub(dt);
        }
        self.trails.retain(|t| !t.time_to_live.is_zero());
        self.trails
            .extend(particles.iter().map(|p| p.create_trail()));
    }

    pub fn get_buffer(&self, device: &Device) -> wgpu::Buffer {
        let raws = self.trails.iter().map(Trail::to_raw).collect::<Vec<_>>();
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("trail buffer"),
            contents: bytemuck::cast_slice(&raws[..]),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    pub fn len(&self) -> u32 {
        self.trails.len() as _
    }
}
