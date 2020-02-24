

#[repr(C)]
pub struct Sphere {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub radius: f32,
    pub mat: u32,
    reserved0: u32,
    reserved1: u32,
    reserved2: u32,
}

#[repr(C)]
pub struct Material {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
    pub ty: u32,
    reserved0: u32,
    reserved1: u32,
    reserved2: u32,
}

#[inline]
pub fn distance(ax: f32, ay: f32, az: f32, bx: f32, by: f32, bz: f32) -> f32 {
    let dx = bx - ax;
    let dy = by - ay;
    let dz = bz - az;
    return (dx * dx + dy * dy + dz * dz).sqrt();
}

#[inline]
fn cross(ax: f32, ay: f32, az: f32, bx: f32, by: f32, bz: f32) -> (f32, f32, f32) {
    let cx = ay * bz - by * az;
    let cy = az * bx - bz * ax;
    let cz = ax * by - bx * ay;
    return (cx, cy, cz);
}

#[inline]
fn length(x: f32, y: f32, z: f32) -> f32 {
    return (x * x + y * y + z * z).sqrt();
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Copy, Clone)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { x, y, z }
    }

    pub fn zero() -> Vec3 {
        Vec3::new(0.0, 0.0, 0.0)
    }
}

impl std::ops::Sub<Vec3> for Vec3 {
    type Output = Vec3;
    fn sub(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl std::ops::Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, v: f32) -> Vec3 {
        Vec3 {
            x: self.x * v,
            y: self.y * v,
            z: self.z * v,
        }
    }
}

impl std::ops::Div<f32> for Vec3 {
    type Output = Vec3;
    fn div(self, v: f32) -> Vec3 {
        Vec3 {
            x: self.x / v,
            y: self.y / v,
            z: self.z / v,
        }
    }
}

impl Vec3 {
    fn cross(self, other: Vec3) -> Vec3 {
        let v = cross(self.x, self.y, self.z, other.x, other.y, other.z);
        Vec3 { x: v.0, y: v.1, z: v.2 }
    }

    fn length(&self) -> f32 {
        length(self.x, self.y, self.z)
    }

    fn normalized(self) -> Vec3 {
        let len = self.length();
        self / len
    }

    fn to_vec4(self, w: f32) -> Vec4 {
        Vec4 { 
            x: self.x, 
            y: self.y, 
            z: self.z, 
            w: w,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Camera {
    origin: Vec4,
    lower_left_corner: Vec4,
    horizontal: Vec4,
    vertical: Vec4,
    u: Vec4,
    v: Vec4,
    w: Vec4,
    lens_radius: Vec4,
}

impl Camera {
    pub fn new(origin: Vec3, look_at: Vec3, up: Vec3, vfov: f32, aspect: f32, aperture: f32, focus_dist: f32) -> Self {
        let lens_radius = aperture / 2.0;
        let theta = vfov * std::f32::consts::PI / 180.0;
        let half_height = (theta / 2.0).tan();
        let half_width = aspect * half_height;
        let w = (origin - look_at).normalized();
        let u = up.cross(w).normalized();
        let v = w.cross(u);
        let lower_left_corner = origin 
            - (u * half_width * focus_dist) 
            - (v * half_height * focus_dist)
            - (w * focus_dist);
        let horizontal = u * (2.0 * half_width * focus_dist);
        let vertical = v * (2.0 * half_height * focus_dist);
        Camera {
            origin: origin.to_vec4(1.0),
            lower_left_corner: lower_left_corner.to_vec4(1.0),
            horizontal: horizontal.to_vec4(1.0),
            vertical: vertical.to_vec4(1.0),
            u: u.to_vec4(0.0),
            v: v.to_vec4(0.0),
            w: w.to_vec4(0.0),
            lens_radius: Vec4 { x: lens_radius, y: 0.0, z: 0.0, w: 0.0 },
        }
    }
}
