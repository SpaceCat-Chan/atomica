pub struct Particle {
    position: cgmath::Point2<f64>,
    velocity: cgmath::Vector2<f64>,
    mass: f64,
    charge: f64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RawParticle {
    position: [f32; 2],
    radius: f32,
    charge: f32,
}

unsafe impl bytemuck::Pod for RawParticle {}
unsafe impl bytemuck::Zeroable for RawParticle {}

impl Particle {
    pub fn new(
        position: cgmath::Point2<f64>,
        velocity: cgmath::Vector2<f64>,
        mass: f64,
        charge: f64,
    ) -> Self {
        Self {
            position,
            velocity,
            mass,
            charge,
        }
    }

    pub fn update(particles: &mut Vec<Particle>, dt: f64) {
        todo!()
    }

    pub fn create_trail(&self) -> crate::particle_trail::Trail {
        crate::particle_trail::Trail::new(
            std::time::Duration::from_secs(3),
            self.position,
            self.mass.sqrt(),
            self.charge,
        )
    }

    pub fn to_raw(&self) -> RawParticle {
        let pos = self.position;
        RawParticle {
            position: [pos.x as _, pos.y as _],
            radius: self.mass.sqrt() as _,
            charge: self.charge as _,
        }
    }
}
