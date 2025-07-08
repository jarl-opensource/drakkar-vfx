use bevy::math::Vec2;
// ====================
// Particles.
// ====================
use bevy_hanabi::graph::{ScalarValue, Value};
use bevy_hanabi::{
    AccelModifier,
    Attribute,
    BoxedModifier,
    ColorOverLifetimeModifier,
    EffectAsset,
    Expr,
    LinearDragModifier,
    Property,
    RadialAccelModifier,
    SetAttributeModifier,
    SetPositionCircleModifier,
    SetPositionCone3dModifier,
    SetPositionSphereModifier,
    SetVelocityCircleModifier,
    SetVelocitySphereModifier,
    SetVelocityTangentModifier,
    ShapeDimension,
    SimulationCondition,
    SimulationSpace,
    SizeOverLifetimeModifier,
    TangentAccelModifier,
    *,
};

// ====================
// Editor.
// ====================
use crate::gui::expr::{XBinaryOp, XBuiltInOp, XExpr, XUnaryOp, XValue};
use crate::gui::inspectors::spawner::SpawnerData;
use crate::gui::models::XDimension;
use crate::gui::models::attr::XAttr;
use crate::gui::models::color::HdrColor;
use crate::gui::models::key_value::{KeyValue, KeyValueEntry};
use crate::gui::models::modifier::{
    XAccelModifier,
    XForceFieldSource,
    XInitModifier,
    XLinearDragModifier,
    XOrientMode,
    XOrientModifier,
    XRadialAccelModifier,
    XRenderModifier,
    XSetAttributeModifier,
    XSetPositionCircleModifier,
    XSetPositionCone3dModifier,
    XSetPositionSphereModifier,
    XSetVelocityCircleModifier,
    XSetVelocitySphereModifier,
    XSetVelocityTangentModifier,
    XTangentAccelModifier,
    XUpdateModifier,
};

pub type TimeVec2 = (f32, Vec2);
pub type TimeColor = (f32, HdrColor);

#[derive(Debug, Clone, PartialEq)]
pub enum XError
{
    UnsupportedScalarType(String),
    UnsupportedVectorType(String),
    UnsupportedValueType(String),
    UnknownBuiltInOperator(String),
    UnknownUnaryOperator(String),
    UnknownBinaryOperator(String),
    ExpressionNotFound(String),
    UnsupportedPropertyType(String),
    UnsupportedModifierType(String),
    UnsupportedAttributeType(String),
    UnknownAttribute(String),
}

impl std::fmt::Display for XError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self {
            XError::UnsupportedScalarType(msg) => {
                write!(f, "Unsupported scalar type: {}", msg)
            }
            XError::UnsupportedVectorType(msg) => {
                write!(f, "Unsupported vector type: {}", msg)
            }
            XError::UnsupportedValueType(msg) => {
                write!(f, "Unsupported value type: {}", msg)
            }
            XError::UnknownBuiltInOperator(msg) => {
                write!(f, "Unknown built-in operator: {}", msg)
            }
            XError::UnknownUnaryOperator(msg) => {
                write!(f, "Unknown unary operator: {}", msg)
            }
            XError::UnknownBinaryOperator(msg) => {
                write!(f, "Unknown binary operator: {}", msg)
            }
            XError::ExpressionNotFound(msg) => write!(f, "Expression not found: {}", msg),
            XError::UnsupportedPropertyType(msg) => {
                write!(f, "Unsupported property type: {}", msg)
            }
            XError::UnsupportedModifierType(msg) => {
                write!(f, "Unsupported modifier type: {}", msg)
            }
            XError::UnsupportedAttributeType(msg) => {
                write!(f, "Unsupported attribute type: {}", msg)
            }
            XError::UnknownAttribute(msg) => {
                write!(f, "Unknown attribute: {}", msg)
            }
        }
    }
}

impl std::error::Error for XError {}

/// Converter for editor models to Hanabi.
pub struct ToHanabi;

impl ToHanabi
{
    pub fn effect_asset(state: &AssetState) -> Result<EffectAsset, XError>
    {
        let mut module = Module::default();

        // Create spawner
        let spawner = Self::create_spawner(state)?;

        // Create properties
        let properties: Result<Vec<_>, _> = state
            .properties
            .iter()
            .map(|entry| Self::property(entry))
            .collect();
        let properties = properties?;

        // Convert init modifiers
        let init_modifiers: Result<Vec<_>, _> = state
            .init_modifiers
            .iter()
            .map(|modifier| Self::init_modifier(modifier, &mut module))
            .collect();
        let init_modifiers = init_modifiers?;

        // Convert update modifiers and extract force fields
        let mut update_modifiers = Vec::new();
        for modifier in &state.update_modifiers {
            // Remove XForceField arm, only handle other update modifiers
            if let Ok(converted_modifier) = Self::update_modifier(modifier, &mut module) {
                update_modifiers.push(converted_modifier);
            }
        }

        // Create a single ForceFieldModifier from all sources
        if !state.force_fields.is_empty() {
            let force_field_modifier =
                Self::create_force_field_modifier(&state.force_fields, &mut module)?;
            update_modifiers.push(force_field_modifier);
        }

        // Convert render modifiers
        let render_modifiers: Result<Vec<_>, _> = state
            .render_modifiers
            .iter()
            .map(|modifier| Self::render_modifier(modifier, &mut module))
            .collect();
        let render_modifiers = render_modifiers?;

        // Create additional render modifiers from size and color over time
        let mut additional_render_modifiers = Vec::new();

        // Add color over time modifier if we have color keyframes
        if !state.color_over_time.is_empty() {
            let mut gradient = Gradient::new();
            for &(t, ref color) in &state.color_over_time {
                let vec4_color = bevy::math::Vec4::new(
                    color.r() * color.i(),
                    color.g() * color.i(),
                    color.b() * color.i(),
                    color.a(),
                );
                gradient.add_key(t, vec4_color);
            }
            additional_render_modifiers
                .push(Box::new(ColorOverLifetimeModifier { gradient }) as BoxedModifier);
        }

        // Add size over time modifier if we have size keyframes
        if !state.size_over_time.is_empty() {
            let mut gradient = Gradient::new();
            for &(t, size) in &state.size_over_time {
                gradient.add_key(t, size);
            }
            additional_render_modifiers.push(Box::new(SizeOverLifetimeModifier {
                gradient,
                screen_space_size: false,
            }) as BoxedModifier);
        }

        // Combine all render modifiers
        let mut all_render_modifiers = render_modifiers;
        all_render_modifiers.extend(additional_render_modifiers);

        // Build the effect asset
        let effect = EffectAsset {
            name: state.name.clone(),
            capacity: state.capacity as u32,
            spawner,
            z_layer_2d: state.z_layer_2d,
            simulation_space: state.simulation_space,
            simulation_condition: state.simulation_condition,
            init_modifiers,
            update_modifiers,
            render_modifiers: all_render_modifiers,
            properties,
            motion_integration: MotionIntegration::default(),
            module,
        };

        Ok(effect)
    }

    fn create_spawner(state: &AssetState) -> Result<Spawner, XError>
    {
        let spawner_data = &state.spawner;

        // Ensure all spawner values are valid for Hanabi
        let num_particles_value = if spawner_data.num_particles < 0.0 {
            1.0 // Default to 1.0 if negative
        } else {
            spawner_data.num_particles
        };
        let num_particles = CpuValue::Single(num_particles_value);

        let spawn_time_value = if spawner_data.spawn_time < 0.0 {
            0.0 // Default to 0.0 if negative
        } else {
            spawner_data.spawn_time
        };
        let spawn_time = CpuValue::Single(spawn_time_value);

        // Ensure period is always positive for Hanabi
        let period_value = if spawner_data.period <= 0.0 {
            1.0 // Default to 1.0 if period is invalid
        } else {
            spawner_data.period
        };
        let period = CpuValue::Single(period_value);

        // Determine spawner type based on values
        let spawner = if spawner_data.period.is_infinite() {
            // Once spawner
            Spawner::once(num_particles, spawner_data.starts_immediately)
        } else if (period_value - spawn_time_value).abs() < f32::EPSILON {
            // Rate spawner (period == spawn_time means continuous)
            Spawner::rate(num_particles)
        } else if spawn_time_value < f32::EPSILON {
            // Burst spawner (spawn_time == 0)
            Spawner::burst(num_particles, period)
        } else {
            // General spawner
            Spawner::new(num_particles, spawn_time, period)
        };

        Ok(spawner.with_starts_active(spawner_data.starts_active))
    }

    fn property(entry: &KeyValueEntry) -> Result<Property, XError>
    {
        let value = match &entry.value {
            KeyValue::Float(f) => Value::Scalar(ScalarValue::Float(*f)),
            KeyValue::Integer(i) => Value::Scalar(ScalarValue::Int(*i)),
            KeyValue::Vec2(v) => Value::Vector((*v).into()),
            KeyValue::Vec3(v) => Value::Vector((*v).into()),
            KeyValue::Color(color) => {
                let vec4 = bevy::math::Vec4::new(
                    color.r() * color.i(),
                    color.g() * color.i(),
                    color.b() * color.i(),
                    color.a(),
                );
                Value::Vector(vec4.into())
            }
        };

        Ok(Property::new(entry.key.clone(), value))
    }

    fn init_modifier(modifier: &XInitModifier, module: &mut Module)
    -> Result<BoxedModifier, XError>
    {
        match modifier {
            XInitModifier::XSetPositionCircle(m) => {
                let center = Self::convert_expr(&m.center, module)?;
                let axis = Self::convert_expr(&m.axis, module)?;
                let radius = Self::convert_expr(&m.radius, module)?;
                let dimension = Self::convert_dimension(&m.dimension);

                Ok(Box::new(SetPositionCircleModifier {
                    center,
                    axis,
                    radius,
                    dimension,
                }))
            }
            XInitModifier::XSetPositionSphere(m) => {
                let center = Self::convert_expr(&m.center, module)?;
                let radius = Self::convert_expr(&m.radius, module)?;
                let dimension = Self::convert_dimension(&m.dimension);

                Ok(Box::new(SetPositionSphereModifier {
                    center,
                    radius,
                    dimension,
                }))
            }
            XInitModifier::XSetPositionCone3d(m) => {
                let height = Self::convert_expr(&m.height, module)?;
                let base_radius = Self::convert_expr(&m.base_radius, module)?;
                let top_radius = Self::convert_expr(&m.top_radius, module)?;
                let dimension = Self::convert_dimension(&m.dimension);

                Ok(Box::new(SetPositionCone3dModifier {
                    height,
                    base_radius,
                    top_radius,
                    dimension,
                }))
            }
            XInitModifier::XSetVelocityCircle(m) => {
                let center = Self::convert_expr(&m.center, module)?;
                let axis = Self::convert_expr(&m.axis, module)?;
                let speed = Self::convert_expr(&m.speed, module)?;

                Ok(Box::new(SetVelocityCircleModifier {
                    center,
                    axis,
                    speed,
                }))
            }
            XInitModifier::XSetVelocitySphere(m) => {
                let center = Self::convert_expr(&m.center, module)?;
                let speed = Self::convert_expr(&m.speed, module)?;

                Ok(Box::new(SetVelocitySphereModifier { center, speed }))
            }
            XInitModifier::XSetVelocityTangent(m) => {
                let origin = Self::convert_expr(&m.center, module)?;
                let axis = module.lit(bevy::math::Vec3::Y); // Default axis since editor model doesn't have axis
                let speed = Self::convert_expr(&m.speed, module)?;

                Ok(Box::new(SetVelocityTangentModifier {
                    origin,
                    axis,
                    speed,
                }))
            }
            XInitModifier::XSetAttribute(m) => {
                let attribute = Self::convert_attr(&m.attr)?;
                let value = Self::convert_expr(&m.value, module)?;

                Ok(Box::new(SetAttributeModifier { attribute, value }))
            }
        }
    }

    fn update_modifier(
        modifier: &XUpdateModifier,
        module: &mut Module,
    ) -> Result<BoxedModifier, XError>
    {
        match modifier {
            XUpdateModifier::XAccel(m) => Ok(Box::new(AccelModifier {
                accel: Self::convert_expr(&m.accel, module)?,
            })),
            XUpdateModifier::XRadialAccel(m) => Ok(Box::new(RadialAccelModifier {
                origin: Self::convert_expr(&m.origin, module)?,
                accel:  Self::convert_expr(&m.accel, module)?,
            })),
            XUpdateModifier::XTangentAccel(m) => Ok(Box::new(TangentAccelModifier {
                origin: Self::convert_expr(&m.origin, module)?,
                axis:   Self::convert_expr(&m.axis, module)?,
                accel:  Self::convert_expr(&m.accel, module)?,
            })),
            XUpdateModifier::XLinearDrag(m) => Ok(Box::new(LinearDragModifier {
                drag: Self::convert_expr(&m.drag, module)?,
            })),
            XUpdateModifier::XSetAttribute(m) => Ok(Box::new(SetAttributeModifier {
                attribute: Self::convert_attr(&m.attr)?,
                value:     Self::convert_expr(&m.value, module)?,
            })),
        }
    }

    fn render_modifier(
        modifier: &XRenderModifier,
        _module: &mut Module,
    ) -> Result<BoxedModifier, XError>
    {
        match modifier {
            XRenderModifier::XOrient(_) => Ok(Box::new(OrientAlongVelocityModifier)),
        }
    }

    fn convert_expr(expr: &XExpr, module: &mut Module) -> Result<ExprHandle, XError>
    {
        use crate::gui::expr::{XBinaryOp, XBuiltInOp, XUnaryOp, XValue};

        match expr {
            XExpr::Lit(value) => {
                let hanabi_value = match value {
                    XValue::Float(f) => Value::Scalar(ScalarValue::Float(*f)),
                    XValue::Integer(i) => Value::Scalar(ScalarValue::Int(*i)),
                    XValue::Vec2(x, y) => Value::Vector(bevy::math::Vec2::new(*x, *y).into()),
                    XValue::Vec3(x, y, z) => {
                        Value::Vector(bevy::math::Vec3::new(*x, *y, *z).into())
                    }
                };
                Ok(module.lit(hanabi_value))
            }
            XExpr::Attr(name) => {
                let attr = Self::convert_attr_by_name(name)?;
                Ok(module.attr(attr))
            }
            XExpr::Prop(name) => Ok(module.prop(name.clone())),
            XExpr::BuiltIn(op) => {
                let hanabi_op = match op {
                    XBuiltInOp::Time => BuiltInOperator::Time,
                    XBuiltInOp::DeltaTime => BuiltInOperator::DeltaTime,
                    XBuiltInOp::Rand => BuiltInOperator::Rand(ValueType::Scalar(ScalarType::Float)),
                    _ => {
                        return Err(XError::UnknownBuiltInOperator(format!(
                            "Unsupported built-in operator: {:?}",
                            op
                        )));
                    }
                };
                Ok(module.builtin(hanabi_op))
            }
            XExpr::Unary { op, expr } => {
                let inner = Self::convert_expr(expr, module)?;
                let hanabi_op = match op {
                    XUnaryOp::Abs => UnaryOperator::Abs,
                    XUnaryOp::All => UnaryOperator::All,
                    XUnaryOp::Any => UnaryOperator::Any,
                    XUnaryOp::Norm => UnaryOperator::Normalize,
                    XUnaryOp::Cos => UnaryOperator::Cos,
                    XUnaryOp::Sin => UnaryOperator::Sin,
                    XUnaryOp::Neg => {
                        return Err(XError::UnknownUnaryOperator(
                            "Neg not supported in Hanabi conversion".to_string(),
                        ));
                    }
                };
                Ok(module.unary(hanabi_op, inner))
            }
            XExpr::Binary { left, op, right } => {
                // Handle special vector construction cases first
                match op {
                    XBinaryOp::Vec2 => {
                        // For vec2, we need to evaluate the expressions and create a vector literal
                        // This is a limitation - we can only create vector literals, not dynamic vector construction
                        let x_val = Self::convert_scalar_expr(left, module)?;
                        let y_val = Self::convert_scalar_expr(right, module)?;
                        let vec2_value = Value::Vector(bevy::math::Vec2::new(x_val, y_val).into());
                        return Ok(module.lit(vec2_value));
                    }
                    XBinaryOp::Vec3 => {
                        // For vec3, we need to handle the nested structure: ((x, y), z)
                        // The left should be a vec2 expression, and right is the z component
                        // We need to extract x and y from the left vec2 expression
                        if let XExpr::Binary {
                            left: vec2_left,
                            op: XBinaryOp::Vec2,
                            right: vec2_right,
                        } = &**left
                        {
                            let x_val = Self::convert_scalar_expr(vec2_left, module)?;
                            let y_val = Self::convert_scalar_expr(vec2_right, module)?;
                            let z_val = Self::convert_scalar_expr(right, module)?;
                            let vec3_value =
                                Value::Vector(bevy::math::Vec3::new(x_val, y_val, z_val).into());
                            return Ok(module.lit(vec3_value));
                        } else {
                            return Err(XError::UnknownBinaryOperator(
                                "Vec3 requires nested Vec2 structure".to_string(),
                            ));
                        }
                    }
                    _ => {} // Continue to normal binary operator handling
                }

                let left_handle = Self::convert_expr(left, module)?;
                let right_handle = Self::convert_expr(right, module)?;
                let hanabi_op = match op {
                    XBinaryOp::Add => BinaryOperator::Add,
                    XBinaryOp::Sub => BinaryOperator::Sub,
                    XBinaryOp::Mul => BinaryOperator::Mul,
                    XBinaryOp::Div => BinaryOperator::Div,
                    XBinaryOp::Lt => BinaryOperator::LessThan,
                    XBinaryOp::Lte => BinaryOperator::LessThanOrEqual,
                    XBinaryOp::Ge => BinaryOperator::GreaterThan,
                    XBinaryOp::Gte => BinaryOperator::GreaterThanOrEqual,
                    XBinaryOp::Min => BinaryOperator::Min,
                    XBinaryOp::Max => BinaryOperator::Max,
                    XBinaryOp::Dot => BinaryOperator::Dot,
                    XBinaryOp::Cross => BinaryOperator::Cross,
                    XBinaryOp::Vec2 | XBinaryOp::Vec3 => {
                        // These cases are handled above, but we need to satisfy the match
                        unreachable!("Vec2 and Vec3 are handled above")
                    }
                    XBinaryOp::Eq => {
                        return Err(XError::UnknownBinaryOperator(
                            "Eq not supported in Hanabi conversion".to_string(),
                        ));
                    }
                    XBinaryOp::Neq => {
                        return Err(XError::UnknownBinaryOperator(
                            "Neq not supported in Hanabi conversion".to_string(),
                        ));
                    }
                };
                Ok(module.binary(hanabi_op, left_handle, right_handle))
            }
        }
    }

    fn convert_dimension(dimension: &XDimension) -> ShapeDimension
    {
        match dimension {
            XDimension::Surface => ShapeDimension::Surface,
            XDimension::Volume => ShapeDimension::Volume,
        }
    }

    fn convert_attr(attr: &XAttr) -> Result<Attribute, XError>
    {
        match attr {
            XAttr::Position => Ok(Attribute::POSITION),
            XAttr::Velocity => Ok(Attribute::VELOCITY),
            XAttr::Age => Ok(Attribute::AGE),
            XAttr::Lifetime => Ok(Attribute::LIFETIME),
            XAttr::Color => Ok(Attribute::COLOR),
            XAttr::Alpha => Ok(Attribute::ALPHA),
            XAttr::Size => Ok(Attribute::SIZE),
            XAttr::Size2 => Ok(Attribute::SIZE2),
            XAttr::AxisX => Ok(Attribute::AXIS_X),
            XAttr::AxisY => Ok(Attribute::AXIS_Y),
            XAttr::AxisZ => Ok(Attribute::AXIS_Z),
        }
    }

    fn convert_attr_by_name(name: &str) -> Result<Attribute, XError>
    {
        match name {
            "position" => Ok(Attribute::POSITION),
            "velocity" => Ok(Attribute::VELOCITY),
            "age" => Ok(Attribute::AGE),
            "lifetime" => Ok(Attribute::LIFETIME),
            "color" => Ok(Attribute::COLOR),
            "alpha" => Ok(Attribute::ALPHA),
            "size" => Ok(Attribute::SIZE),
            "size2" => Ok(Attribute::SIZE2),
            "axis_x" => Ok(Attribute::AXIS_X),
            "axis_y" => Ok(Attribute::AXIS_Y),
            "axis_z" => Ok(Attribute::AXIS_Z),
            _ => Err(XError::UnknownAttribute(name.to_string())),
        }
    }

    fn convert_scalar_expr(expr: &XExpr, module: &mut Module) -> Result<f32, XError>
    {
        let expr_handle = Self::convert_expr(expr, module)?;
        match module.get(expr_handle) {
            Some(Expr::Literal(lit)) => match &lit.value {
                Value::Scalar(scalar) => match scalar {
                    ScalarValue::Float(f) => Ok(*f),
                    ScalarValue::Int(i) => Ok(*i as f32),
                    _ => Err(XError::UnsupportedScalarType(format!("{:?}", scalar))),
                },
                _ => Err(XError::UnsupportedValueType(format!("{:?}", lit.value))),
            },
            _ => Err(XError::ExpressionNotFound(
                "Expected literal expression".to_string(),
            )),
        }
    }

    fn create_force_field_modifier(
        sources: &[XForceFieldSource],
        module: &mut Module,
    ) -> Result<BoxedModifier, XError>
    {
        // Convert Vec<ForceFieldSource> to array format required by Hanabi
        let mut source_array =
            [modifier::ForceFieldSource::default(); modifier::ForceFieldSource::MAX_SOURCES];

        for (i, source) in sources
            .iter()
            .take(modifier::ForceFieldSource::MAX_SOURCES)
            .enumerate()
        {
            // Convert position expression to Vec3
            let position_expr = Self::convert_expr(&source.position, module)?;
            let position = match module.get(position_expr) {
                Some(Expr::Literal(lit)) => match &lit.value {
                    Value::Vector(vector) => match vector.vector_type() {
                        VectorType::VEC3F => {
                            let values = vector.get_all::<f32>();
                            bevy::math::Vec3::new(values[0], values[1], values[2])
                        }
                        _ => bevy::math::Vec3::ZERO,
                    },
                    _ => bevy::math::Vec3::ZERO,
                },
                _ => bevy::math::Vec3::ZERO,
            };

            // Convert scalar expressions to f32 values
            let max_radius = Self::convert_scalar_expr(&source.max_radius, module)?;
            let min_radius = Self::convert_scalar_expr(&source.min_radius, module)?;
            let mass = Self::convert_scalar_expr(&source.mass, module)?;
            let force_exponent = Self::convert_scalar_expr(&source.force_exponent, module)?;

            source_array[i] = modifier::ForceFieldSource {
                position,
                max_radius,
                min_radius,
                mass,
                force_exponent,
                conform_to_sphere: source.conform_to_sphere,
            };
        }

        Ok(Box::new(modifier::ForceFieldModifier {
            sources: source_array,
        }))
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AssetState
{
    pub name:                 String,
    pub capacity:             i32,
    pub z_layer_2d:           f32,
    pub simulation_space:     SimulationSpace,
    pub simulation_condition: SimulationCondition,

    pub spawner: SpawnerData,

    pub size_over_time:  Vec<TimeVec2>,
    pub color_over_time: Vec<TimeColor>,

    pub init_modifiers:   Vec<XInitModifier>,
    pub update_modifiers: Vec<XUpdateModifier>,
    pub render_modifiers: Vec<XRenderModifier>,

    pub force_fields: Vec<XForceFieldSource>,

    pub properties: Vec<KeyValueEntry>,
}

/// Converter for Hanabi to editor models.
pub struct FromHanabi;

impl FromHanabi
{
    pub fn asset_state(asset: &EffectAsset) -> Result<AssetState, XError>
    {
        let spawner = &asset.spawner;
        let num_particles = spawner.num_particles.range()[0];
        let spawn_time = spawner.spawn_time.range()[0];
        let period = spawner.period.range()[0];
        let properties = asset
            .properties
            .iter()
            .filter_map(|prop| Self::prop(prop).ok())
            .collect();

        let init_modifiers = asset
            .init_modifiers
            .iter()
            .filter_map(|modifier| Self::init_modifier(modifier, &asset.module).ok())
            .collect();

        // Extract update modifiers and force fields
        let mut update_modifiers = Vec::new();
        let mut force_fields: Vec<XForceFieldSource> = Vec::new();

        for modifier in &asset.update_modifiers {
            if let Ok(converted_modifier) = Self::update_modifier(modifier, &asset.module) {
                update_modifiers.push(converted_modifier);
            }

            // Check if this is a force field modifier and extract sources
            if let Ok(sources) = Self::extract_force_field_sources(modifier) {
                force_fields.extend(sources);
            }
        }

        // Extract render modifiers and populate size_over_time and color_over_time
        let mut size_over_time = Vec::new();
        let mut color_over_time = Vec::new();
        let mut render_modifiers = Vec::new();

        for modifier in &asset.render_modifiers {
            // Try to extract as orient modifier first
            if let Ok(orient_modifier) = Self::render_modifier_from_hanabi(modifier, &asset.module)
            {
                render_modifiers.push(orient_modifier);
            } else {
                // Fall back to legacy render modifiers (size/color over time)
                if let Ok((color_keys, size_keys)) = Self::render_modifier_legacy(modifier) {
                    color_over_time.extend(color_keys);
                    size_over_time.extend(size_keys);
                }
            }
        }

        Ok(AssetState {
            name: asset.name.clone(),
            capacity: asset.capacity as i32,
            z_layer_2d: asset.z_layer_2d,
            simulation_space: asset.simulation_space,
            simulation_condition: asset.simulation_condition,

            spawner: SpawnerData {
                num_particles,
                spawn_time,
                period,
                starts_active: spawner.starts_active,
                starts_immediately: spawner.starts_immediately,
            },

            size_over_time,
            color_over_time,

            init_modifiers,
            update_modifiers,
            render_modifiers,

            force_fields,

            properties,
        })
    }

    fn prop(prop: &Property) -> Result<KeyValueEntry, XError>
    {
        let prop_value = prop.default_value();
        let key_value = match prop_value {
            &Value::Scalar(scalar) => match scalar {
                ScalarValue::Float(f) => Ok(KeyValue::Float(f)),
                ScalarValue::Int(i) => Ok(KeyValue::Integer(i)),
                ScalarValue::Bool(b) => Ok(KeyValue::Integer(if b { 1 } else { 0 })),
                _ => Err(XError::UnsupportedPropertyType(format!(
                    "Scalar type: {:?}",
                    scalar
                ))),
            },
            &Value::Vector(vector) => match vector.vector_type() {
                VectorType::VEC2F => {
                    let values = vector.get_all::<f32>();
                    Ok(KeyValue::Vec2(Vec2::new(values[0], values[1])))
                }
                VectorType::VEC3F => {
                    let values = vector.get_all::<f32>();
                    Ok(KeyValue::Vec3(bevy::math::Vec3::new(
                        values[0], values[1], values[2],
                    )))
                }
                VectorType::VEC4F => {
                    let values = vector.get_all::<f32>();
                    Ok(KeyValue::Color(HdrColor::new(
                        values[0], values[1], values[2], values[3], 1.0,
                    )))
                }
                _ => Err(XError::UnsupportedPropertyType(format!(
                    "Vector type: {:?}",
                    vector.vector_type()
                ))),
            },
            _ => Err(XError::UnsupportedPropertyType(format!(
                "Value type: {:?}",
                prop_value
            ))),
        }?;

        Ok(KeyValueEntry::new(prop.name().to_string(), key_value))
    }

    fn init_modifier(bm: &BoxedModifier, m: &Module) -> Result<XInitModifier, XError>
    {
        // Check if this is an init modifier first
        if bm.as_init().is_none() {
            return Err(XError::UnsupportedModifierType(
                "Not an init modifier".to_string(),
            ));
        }

        let any_mod = bm.as_any();
        if let Some(c) = any_mod.downcast_ref::<SetPositionCircleModifier>() {
            Ok(XInitModifier::XSetPositionCircle(
                XSetPositionCircleModifier {
                    center:    Self::xepr(m.get(c.center).unwrap(), m)?,
                    axis:      Self::xepr(m.get(c.axis).unwrap(), m)?,
                    radius:    Self::xepr(m.get(c.radius).unwrap(), m)?,
                    dimension: Self::convert_shape_dimension(c.dimension),
                },
            ))
        } else if let Some(s) = any_mod.downcast_ref::<SetPositionSphereModifier>() {
            Ok(XInitModifier::XSetPositionSphere(
                XSetPositionSphereModifier {
                    center:    Self::xepr(m.get(s.center).unwrap(), m)?,
                    radius:    Self::xepr(m.get(s.radius).unwrap(), m)?,
                    dimension: Self::convert_shape_dimension(s.dimension),
                },
            ))
        } else if let Some(c) = any_mod.downcast_ref::<SetPositionCone3dModifier>() {
            Ok(XInitModifier::XSetPositionCone3d(
                XSetPositionCone3dModifier {
                    height:      Self::xepr(m.get(c.height).unwrap(), m)?,
                    base_radius: Self::xepr(m.get(c.base_radius).unwrap(), m)?,
                    top_radius:  Self::xepr(m.get(c.top_radius).unwrap(), m)?,
                    dimension:   Self::convert_shape_dimension(c.dimension),
                },
            ))
        } else if let Some(v) = any_mod.downcast_ref::<SetVelocityCircleModifier>() {
            Ok(XInitModifier::XSetVelocityCircle(
                XSetVelocityCircleModifier {
                    center: Self::xepr(m.get(v.center).unwrap(), m)?,
                    axis:   Self::xepr(m.get(v.axis).unwrap(), m)?,
                    speed:  Self::xepr(m.get(v.speed).unwrap(), m)?,
                },
            ))
        } else if let Some(v) = any_mod.downcast_ref::<SetVelocitySphereModifier>() {
            Ok(XInitModifier::XSetVelocitySphere(
                XSetVelocitySphereModifier {
                    center: Self::xepr(m.get(v.center).unwrap(), m)?,
                    speed:  Self::xepr(m.get(v.speed).unwrap(), m)?,
                },
            ))
        } else if let Some(v) = any_mod.downcast_ref::<SetVelocityTangentModifier>() {
            Ok(XInitModifier::XSetVelocityTangent(
                XSetVelocityTangentModifier {
                    center: Self::xepr(m.get(v.origin).unwrap(), m)?,
                    speed:  Self::xepr(m.get(v.speed).unwrap(), m)?,
                },
            ))
        } else if let Some(a) = any_mod.downcast_ref::<SetAttributeModifier>() {
            Ok(XInitModifier::XSetAttribute(XSetAttributeModifier {
                attr:  Self::attr(&a.attribute)?,
                value: Self::xepr(m.get(a.value).unwrap(), m)?,
            }))
        } else {
            Err(XError::UnsupportedModifierType(
                "Unknown init modifier type".to_string(),
            ))
        }
    }

    fn update_modifier(bm: &BoxedModifier, m: &Module) -> Result<XUpdateModifier, XError>
    {
        if bm.as_update().is_none() {
            return Err(XError::UnsupportedModifierType(
                "Not an update modifier".to_string(),
            ));
        }

        let any_mod = bm.as_any();
        if let Some(l) = any_mod.downcast_ref::<LinearDragModifier>() {
            Ok(XUpdateModifier::XLinearDrag(XLinearDragModifier {
                drag: Self::xepr(m.get(l.drag).unwrap(), m)?,
            }))
        } else if let Some(a) = any_mod.downcast_ref::<AccelModifier>() {
            Ok(XUpdateModifier::XAccel(XAccelModifier {
                accel: Self::xepr(m.get(a.accel).unwrap(), m)?,
            }))
        } else if let Some(r) = any_mod.downcast_ref::<RadialAccelModifier>() {
            Ok(XUpdateModifier::XRadialAccel(XRadialAccelModifier {
                origin: Self::xepr(m.get(r.origin).unwrap(), m)?,
                accel:  Self::xepr(m.get(r.accel).unwrap(), m)?,
            }))
        } else if let Some(t) = any_mod.downcast_ref::<TangentAccelModifier>() {
            Ok(XUpdateModifier::XTangentAccel(XTangentAccelModifier {
                origin: Self::xepr(m.get(t.origin).unwrap(), m)?,
                axis:   Self::xepr(m.get(t.axis).unwrap(), m)?,
                accel:  Self::xepr(m.get(t.accel).unwrap(), m)?,
            }))
        } else if let Some(attr_mod) = any_mod.downcast_ref::<SetAttributeModifier>() {
            Ok(XUpdateModifier::XSetAttribute(XSetAttributeModifier {
                attr:  Self::attr(&attr_mod.attribute)?,
                value: Self::xepr(m.get(attr_mod.value).unwrap(), m)?,
            }))
        } else {
            Err(XError::UnsupportedModifierType(
                "Unknown update modifier type".to_string(),
            ))
        }
    }

    fn convert_shape_dimension(dimension: ShapeDimension) -> XDimension
    {
        match dimension {
            ShapeDimension::Surface => XDimension::Surface,
            ShapeDimension::Volume => XDimension::Volume,
        }
    }

    fn attr(attr: &Attribute) -> Result<XAttr, XError>
    {
        if *attr == Attribute::POSITION {
            Ok(XAttr::Position)
        } else if *attr == Attribute::VELOCITY {
            Ok(XAttr::Velocity)
        } else if *attr == Attribute::AGE {
            Ok(XAttr::Age)
        } else if *attr == Attribute::LIFETIME {
            Ok(XAttr::Lifetime)
        } else if *attr == Attribute::COLOR {
            Ok(XAttr::Color)
        } else if *attr == Attribute::ALPHA {
            Ok(XAttr::Alpha)
        } else if *attr == Attribute::SIZE {
            Ok(XAttr::Size)
        } else if *attr == Attribute::SIZE2 {
            Ok(XAttr::Size2)
        } else if *attr == Attribute::AXIS_X {
            Ok(XAttr::AxisX)
        } else if *attr == Attribute::AXIS_Y {
            Ok(XAttr::AxisY)
        } else if *attr == Attribute::AXIS_Z {
            Ok(XAttr::AxisZ)
        } else {
            Err(XError::UnknownAttribute(format!("{:?}", attr)))
        }
    }

    pub fn xepr(expr: &Expr, m: &Module) -> Result<XExpr, XError>
    {
        match expr {
            Expr::Literal(lit) => Self::convert_literal_value(&lit.value),
            Expr::Attribute(attr) => Ok(XExpr::Attr(attr.attr.name().to_string())),
            Expr::Property(prop) => Ok(XExpr::Prop(prop.property_name.clone())),
            Expr::BuiltIn(builtin) => Ok(XExpr::BuiltIn(Self::convert_builtin_operator(
                &builtin.operator,
            )?)),
            Expr::Unary { op, expr } => {
                let inner_expr = m.get(*expr).ok_or_else(|| {
                    XError::ExpressionNotFound(format!("Unary expression handle {:?}", expr))
                })?;
                let converted_inner = Self::xepr(inner_expr, m)?;
                Ok(XExpr::unary(
                    Self::convert_unary_operator(op.clone())?,
                    converted_inner,
                ))
            }
            Expr::Binary { op, left, right } => {
                let left_expr = m.get(*left).ok_or_else(|| {
                    XError::ExpressionNotFound(format!("Binary left expression handle {:?}", left))
                })?;
                let right_expr = m.get(*right).ok_or_else(|| {
                    XError::ExpressionNotFound(format!(
                        "Binary right expression handle {:?}",
                        right
                    ))
                })?;
                let converted_left = Self::xepr(left_expr, m)?;
                let converted_right = Self::xepr(right_expr, m)?;
                Ok(XExpr::binary(
                    converted_left,
                    Self::convert_binary_operator(op.clone())?,
                    converted_right,
                ))
            }
        }
    }

    fn convert_literal_value(lit: &Value) -> Result<XExpr, XError>
    {
        match lit {
            Value::Scalar(scalar) => match scalar {
                ScalarValue::Float(f) => Ok(XExpr::lit(*f)),
                ScalarValue::Int(i) => Ok(XExpr::lit(*i)),
                _ => Err(XError::UnsupportedScalarType(format!("{:?}", scalar))),
            },
            Value::Vector(vector) => match vector.vector_type() {
                VectorType::VEC2F => {
                    let values = vector.get_all::<f32>();
                    Ok(XExpr::lit(XValue::vec2(values[0], values[1])))
                }
                VectorType::VEC3F => {
                    let values = vector.get_all::<f32>();
                    Ok(XExpr::lit(XValue::vec3(values[0], values[1], values[2])))
                }
                _ => Err(XError::UnsupportedVectorType(format!(
                    "{:?}",
                    vector.vector_type()
                ))),
            },
            _ => Err(XError::UnsupportedValueType(format!("{:?}", lit))),
        }
    }

    fn convert_builtin_operator(op: &BuiltInOperator) -> Result<XBuiltInOp, XError>
    {
        match op {
            BuiltInOperator::Time => Ok(XBuiltInOp::Time),
            BuiltInOperator::DeltaTime => Ok(XBuiltInOp::DeltaTime),
            BuiltInOperator::Rand(_) => Ok(XBuiltInOp::Rand),
        }
    }

    fn convert_unary_operator(op: UnaryOperator) -> Result<XUnaryOp, XError>
    {
        match op {
            UnaryOperator::Abs => Ok(XUnaryOp::Abs),
            UnaryOperator::All => Ok(XUnaryOp::All),
            UnaryOperator::Any => Ok(XUnaryOp::Any),
            UnaryOperator::Normalize => Ok(XUnaryOp::Norm),
            UnaryOperator::Cos => Ok(XUnaryOp::Cos),
            UnaryOperator::Sin => Ok(XUnaryOp::Sin),
        }
    }

    fn convert_binary_operator(op: BinaryOperator) -> Result<XBinaryOp, XError>
    {
        match op {
            BinaryOperator::Add => Ok(XBinaryOp::Add),
            BinaryOperator::Sub => Ok(XBinaryOp::Sub),
            BinaryOperator::Mul => Ok(XBinaryOp::Mul),
            BinaryOperator::Div => Ok(XBinaryOp::Div),
            BinaryOperator::LessThan => Ok(XBinaryOp::Lt),
            BinaryOperator::LessThanOrEqual => Ok(XBinaryOp::Lte),
            BinaryOperator::GreaterThan => Ok(XBinaryOp::Ge),
            BinaryOperator::GreaterThanOrEqual => Ok(XBinaryOp::Gte),
            BinaryOperator::Min => Ok(XBinaryOp::Min),
            BinaryOperator::Max => Ok(XBinaryOp::Max),
            BinaryOperator::Dot => Ok(XBinaryOp::Dot),
            BinaryOperator::Cross => Ok(XBinaryOp::Cross),
            BinaryOperator::UniformRand => Err(XError::UnknownBinaryOperator(
                "UniformRand not supported in editor expressions".to_string(),
            )),
        }
    }

    fn render_modifier_from_hanabi(
        bm: &BoxedModifier,
        _m: &Module,
    ) -> Result<XRenderModifier, XError>
    {
        let any_mod = bm.as_any();

        if let Some(_) = any_mod.downcast_ref::<OrientAlongVelocityModifier>() {
            Ok(XRenderModifier::XOrient(XOrientModifier {
                mode: XOrientMode::AlongVelocity,
            }))
        } else {
            Err(XError::UnsupportedModifierType(
                "Unknown render modifier type".to_string(),
            ))
        }
    }

    fn render_modifier_legacy(bm: &BoxedModifier)
    -> Result<(Vec<TimeColor>, Vec<TimeVec2>), XError>
    {
        if bm.as_render().is_none() {
            return Err(XError::UnsupportedModifierType(
                "Not a render modifier".to_string(),
            ));
        }

        let any_mod = bm.as_any();
        let mut color_keys = Vec::new();
        let mut size_keys = Vec::new();

        if let Some(c) = any_mod.downcast_ref::<ColorOverLifetimeModifier>() {
            for key in c.gradient.keys() {
                let color = key.value;
                let max_rgb = color.x.max(color.y).max(color.z);

                let hdr_color = if max_rgb <= 1.0 {
                    HdrColor::new(color.x, color.y, color.z, color.w, 1.0)
                } else {
                    let intensity = max_rgb;
                    HdrColor::new(
                        color.x / max_rgb,
                        color.y / max_rgb,
                        color.z / max_rgb,
                        color.w,
                        intensity,
                    )
                };
                color_keys.push((key.ratio, hdr_color));
            }
        } else if let Some(s) = any_mod.downcast_ref::<SizeOverLifetimeModifier>() {
            // Convert gradient keys to size keyframes
            for key in s.gradient.keys() {
                size_keys.push((key.ratio, key.value));
            }
        } else {
            return Err(XError::UnsupportedModifierType(
                "Unknown render modifier type".to_string(),
            ));
        }

        Ok((color_keys, size_keys))
    }

    fn extract_force_field_sources(bm: &BoxedModifier) -> Result<Vec<XForceFieldSource>, XError>
    {
        let any_mod = bm.as_any();

        if let Some(force_field) = any_mod.downcast_ref::<modifier::ForceFieldModifier>() {
            let mut sources = Vec::new();

            for source in &force_field.sources {
                // Only include sources with non-zero mass (active sources)
                if source.mass > 0.0 {
                    sources.push(XForceFieldSource {
                        position:          Self::xepr(
                            &Expr::Literal(LiteralExpr {
                                value: Value::Vector(source.position.into()),
                            }),
                            &Module::default(),
                        )?,
                        max_radius:        Self::xepr(
                            &Expr::Literal(LiteralExpr {
                                value: Value::Scalar(ScalarValue::Float(source.max_radius)),
                            }),
                            &Module::default(),
                        )?,
                        min_radius:        Self::xepr(
                            &Expr::Literal(LiteralExpr {
                                value: Value::Scalar(ScalarValue::Float(source.min_radius)),
                            }),
                            &Module::default(),
                        )?,
                        mass:              Self::xepr(
                            &Expr::Literal(LiteralExpr {
                                value: Value::Scalar(ScalarValue::Float(source.mass)),
                            }),
                            &Module::default(),
                        )?,
                        force_exponent:    Self::xepr(
                            &Expr::Literal(LiteralExpr {
                                value: Value::Scalar(ScalarValue::Float(source.force_exponent)),
                            }),
                            &Module::default(),
                        )?,
                        conform_to_sphere: source.conform_to_sphere,
                    });
                }
            }

            Ok(sources)
        } else {
            Ok(vec![])
        }
    }
}
