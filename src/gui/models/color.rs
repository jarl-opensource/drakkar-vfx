use bevy::prelude::{Vec3, Vec4};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HdrColor
{
    r: f32,
    g: f32,
    b: f32,
    a: f32,
    i: f32,
}

impl HdrColor
{
    #[inline]
    pub fn new(r: f32, g: f32, b: f32, a: f32, i: f32) -> Self
    {
        Self { r, g, b, a, i }
    }

    #[inline]
    pub fn from_gpui(rgba: gpui::Rgba, intensity: f32) -> Self
    {
        Self {
            r: rgba.r,
            g: rgba.g,
            b: rgba.b,
            a: rgba.a,
            i: intensity,
        }
    }

    #[inline]
    pub fn from_gpui_default(rgba: gpui::Rgba) -> Self
    {
        Self::from_gpui(rgba, 1.0)
    }

    #[inline]
    pub fn r(&self) -> f32
    {
        self.r
    }
    #[inline]
    pub fn g(&self) -> f32
    {
        self.g
    }
    #[inline]
    pub fn b(&self) -> f32
    {
        self.b
    }
    #[inline]
    pub fn a(&self) -> f32
    {
        self.a
    }
    #[inline]
    pub fn i(&self) -> f32
    {
        self.i
    }

    #[inline]
    pub fn set_r(&mut self, r: f32)
    {
        self.r = r;
    }
    #[inline]
    pub fn set_g(&mut self, g: f32)
    {
        self.g = g;
    }
    #[inline]
    pub fn set_b(&mut self, b: f32)
    {
        self.b = b;
    }
    #[inline]
    pub fn set_a(&mut self, a: f32)
    {
        self.a = a;
    }
    #[inline]
    pub fn set_i(&mut self, i: f32)
    {
        self.i = i;
    }

    #[inline]
    pub fn with_r(mut self, r: f32) -> Self
    {
        self.r = r;
        self
    }
    #[inline]
    pub fn with_g(mut self, g: f32) -> Self
    {
        self.g = g;
        self
    }
    #[inline]
    pub fn with_b(mut self, b: f32) -> Self
    {
        self.b = b;
        self
    }
    #[inline]
    pub fn with_a(mut self, a: f32) -> Self
    {
        self.a = a;
        self
    }
    #[inline]
    pub fn with_i(mut self, i: f32) -> Self
    {
        self.i = i;
        self
    }

    #[inline]
    pub fn base_color(&self) -> gpui::Rgba
    {
        gpui::Rgba {
            r: self.r,
            g: self.g,
            b: self.b,
            a: 1.0,
        }
    }

    #[inline]
    pub fn with_alpha(&self) -> gpui::Rgba
    {
        gpui::Rgba {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }

    #[inline]
    pub fn vec3(&self) -> Vec3
    {
        Vec3::new(self.r, self.g, self.b) * self.i
    }

    #[inline]
    pub fn vec4(&self) -> Vec4
    {
        (Vec3::new(self.r, self.g, self.b) * self.i).extend(self.a)
    }

    #[inline]
    pub fn bevy(&self) -> bevy::render::color::Color
    {
        bevy::render::color::Color::rgba(self.r * self.i, self.g * self.i, self.b * self.i, self.a)
    }

    #[inline]
    pub fn gpui(&self) -> gpui::Rgba
    {
        gpui::Rgba {
            r: self.r * self.i,
            g: self.g * self.i,
            b: self.b * self.i,
            a: self.a,
        }
    }
}

impl Default for HdrColor
{
    fn default() -> Self
    {
        Self::new(1.0, 1.0, 1.0, 1.0, 1.0)
    }
}
