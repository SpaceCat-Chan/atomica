use cgmath::num_traits::Pow;

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

fn lennard_jones_force(d: f64) -> f64 {
    const EPSILON: f64 = 100.0;
    const TURBO: f64 = 3.0;
    (24.0 * EPSILON / d * d) * ((2.0 * TURBO / d).pow(12.0) - (TURBO / d).pow(6))
}

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
        let mut forces = Vec::new();
        forces.resize(particles.len(), cgmath::vec2(0.0, 0.0));

        //PUT EKSEMPEL HER

        //ENDE AF EKSEMPEL

        for (particle, force) in particles.iter_mut().zip(forces.iter()) {
            particle.velocity += (force / particle.mass) * dt;
            particle.position += particle.velocity * dt;
        }
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
