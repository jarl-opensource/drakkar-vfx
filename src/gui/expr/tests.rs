#[cfg(test)]
mod tests
{
    use std::collections::HashMap;

    use crate::gui::expr::xexpr::XExpr;
    use crate::gui::expr::xop::{XBinaryOp, XBuiltInOp, XUnaryOp};
    use crate::gui::expr::xparser::*;
    use crate::gui::expr::xval::{XExprReturnType, XValue};

    #[test]
    fn test_value_constructors()
    {
        assert_eq!(XValue::float(1.5), XValue::Float(1.5));
        assert_eq!(XValue::int(42), XValue::Integer(42));
        assert_eq!(XValue::vec2(1.0, 2.0), XValue::Vec2(1.0, 2.0));
        assert_eq!(XValue::vec3(1.0, 2.0, 3.0), XValue::Vec3(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_value_display()
    {
        assert_eq!(XValue::Float(1.5).to_string(), "1.5");
        assert_eq!(XValue::Integer(42).to_string(), "42");
        assert_eq!(XValue::Vec2(1.0, 2.0).to_string(), "vec2(1.0, 2.0)");
        assert_eq!(
            XValue::Vec3(1.0, 2.0, 3.0).to_string(),
            "vec3(1.0, 2.0, 3.0)"
        );
    }

    #[test]
    fn test_basic_expr_construction()
    {
        let expr = XExpr::binary(XExpr::lit(1.0f32), XBinaryOp::Add, XExpr::lit(2.0f32));
        assert_eq!(expr.to_string(), "1.0 + 2.0");
    }

    #[test]
    fn test_parse_literals()
    {
        assert_eq!(XExpr::parse("42").unwrap(), XExpr::lit(XValue::Integer(42)));
        assert_eq!(
            XExpr::parse("3.14").unwrap(),
            XExpr::lit(XValue::Float(3.14))
        );
    }

    #[test]
    fn test_parse_binary_operations()
    {
        assert_eq!(
            XExpr::parse("1 + 2").unwrap(),
            XExpr::binary(
                XExpr::lit(XValue::Integer(1)),
                XBinaryOp::Add,
                XExpr::lit(XValue::Integer(2))
            )
        );

        assert_eq!(
            XExpr::parse("3.0 * 4.5").unwrap(),
            XExpr::binary(
                XExpr::lit(XValue::Float(3.0)),
                XBinaryOp::Mul,
                XExpr::lit(XValue::Float(4.5))
            )
        );
    }

    #[test]
    fn test_parse_unary_operations()
    {
        assert_eq!(
            XExpr::parse("-5").unwrap(),
            XExpr::unary(XUnaryOp::Neg, XExpr::lit(XValue::Integer(5)))
        );

        assert_eq!(
            XExpr::parse("sin(1.0)").unwrap(),
            XExpr::sin(XExpr::lit(XValue::Float(1.0)))
        );

        assert_eq!(
            XExpr::parse("cos(time)").unwrap(),
            XExpr::cos(XExpr::builtin(XBuiltInOp::Time))
        );

        assert_eq!(
            XExpr::parse("abs(-3.14)").unwrap(),
            XExpr::abs(XExpr::unary(XUnaryOp::Neg, XExpr::lit(XValue::Float(3.14))))
        );

        assert_eq!(
            XExpr::parse("norm(velocity)").unwrap(),
            XExpr::norm(XExpr::attr("velocity"))
        );
    }

    #[test]
    fn test_parse_builtin_operators()
    {
        assert_eq!(
            XExpr::parse("time").unwrap(),
            XExpr::builtin(XBuiltInOp::Time)
        );

        assert_eq!(
            XExpr::parse("delta_time").unwrap(),
            XExpr::builtin(XBuiltInOp::DeltaTime)
        );

        assert_eq!(
            XExpr::parse("rand").unwrap(),
            XExpr::builtin(XBuiltInOp::Rand)
        );

        assert_eq!(
            XExpr::parse("alpha_cutoff").unwrap(),
            XExpr::builtin(XBuiltInOp::AlphaCutoff)
        );

        assert_eq!(
            XExpr::parse("particle_id").unwrap(),
            XExpr::builtin(XBuiltInOp::ParticleId)
        );
    }

    #[test]
    fn test_parse_attributes_and_properties()
    {
        assert_eq!(XExpr::parse("velocity").unwrap(), XExpr::attr("velocity"));

        assert_eq!(
            XExpr::parse("attr(\"position\")").unwrap(),
            XExpr::attr("position")
        );

        assert_eq!(
            XExpr::parse("prop(\"speed\")").unwrap(),
            XExpr::prop("speed")
        );

        assert_eq!(
            XExpr::parse("attr(lifetime)").unwrap(),
            XExpr::attr("lifetime")
        );
    }

    #[test]
    fn test_parse_vector_constructors()
    {
        assert_eq!(
            XExpr::parse("vec2(1.0, 2.0)").unwrap(),
            XExpr::lit(XValue::vec2(1.0, 2.0))
        );

        assert_eq!(
            XExpr::parse("vec3(1.0, 2.0, 3.0)").unwrap(),
            XExpr::lit(XValue::vec3(1.0, 2.0, 3.0))
        );
    }

    #[test]
    fn test_parse_vector_constructors_with_negative_literals()
    {
        // Test vec2 with negative literals
        assert_eq!(
            XExpr::parse("vec2(-1.0, -2.0)").unwrap(),
            XExpr::lit(XValue::vec2(-1.0, -2.0))
        );

        // Test vec3 with negative literals
        assert_eq!(
            XExpr::parse("vec3(-0.0, -15.0, -0.0)").unwrap(),
            XExpr::lit(XValue::vec3(-0.0, -15.0, -0.0))
        );

        // Test the specific expression that's failing
        let result = XExpr::parse("(vec3(-0.0, -15.0, -0.0) - (rand * vec3(0.0, 15.0, 0.0)))");
        assert!(
            result.is_ok(),
            "Should parse expression with negative literals in vec3: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_binary_functions()
    {
        assert_eq!(
            XExpr::parse("dot(a, b)").unwrap(),
            XExpr::binary(XExpr::attr("a"), XBinaryOp::Dot, XExpr::attr("b"))
        );

        assert_eq!(
            XExpr::parse("cross(vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0))").unwrap(),
            XExpr::binary(
                XExpr::lit(XValue::vec3(1.0, 0.0, 0.0)),
                XBinaryOp::Cross,
                XExpr::lit(XValue::vec3(0.0, 1.0, 0.0))
            )
        );

        assert_eq!(
            XExpr::parse("min(x, y)").unwrap(),
            XExpr::binary(XExpr::attr("x"), XBinaryOp::Min, XExpr::attr("y"))
        );

        assert_eq!(
            XExpr::parse("max(5.0, 10.0)").unwrap(),
            XExpr::binary(
                XExpr::lit(XValue::Float(5.0)),
                XBinaryOp::Max,
                XExpr::lit(XValue::Float(10.0))
            )
        );
    }

    #[test]
    fn test_parse_comparison_operators()
    {
        assert_eq!(
            XExpr::parse("x < y").unwrap(),
            XExpr::binary(XExpr::attr("x"), XBinaryOp::Lt, XExpr::attr("y"))
        );

        assert_eq!(
            XExpr::parse("a <= b").unwrap(),
            XExpr::binary(XExpr::attr("a"), XBinaryOp::Lte, XExpr::attr("b"))
        );

        assert_eq!(
            XExpr::parse("1.0 > 0.5").unwrap(),
            XExpr::binary(
                XExpr::lit(XValue::Float(1.0)),
                XBinaryOp::Ge,
                XExpr::lit(XValue::Float(0.5))
            )
        );

        assert_eq!(
            XExpr::parse("x >= y").unwrap(),
            XExpr::binary(XExpr::attr("x"), XBinaryOp::Gte, XExpr::attr("y"))
        );

        assert_eq!(
            XExpr::parse("a == b").unwrap(),
            XExpr::binary(XExpr::attr("a"), XBinaryOp::Eq, XExpr::attr("b"))
        );

        assert_eq!(
            XExpr::parse("x != y").unwrap(),
            XExpr::binary(XExpr::attr("x"), XBinaryOp::Neq, XExpr::attr("y"))
        );
    }

    #[test]
    fn test_parse_complex_expressions()
    {
        assert_eq!(
            XExpr::parse("1 + 2 * 3").unwrap(),
            XExpr::binary(
                XExpr::lit(XValue::Integer(1)),
                XBinaryOp::Add,
                XExpr::binary(
                    XExpr::lit(XValue::Integer(2)),
                    XBinaryOp::Mul,
                    XExpr::lit(XValue::Integer(3))
                )
            )
        );

        assert_eq!(
            XExpr::parse("(1 + 2) * 3").unwrap(),
            XExpr::binary(
                XExpr::binary(
                    XExpr::lit(XValue::Integer(1)),
                    XBinaryOp::Add,
                    XExpr::lit(XValue::Integer(2))
                ),
                XBinaryOp::Mul,
                XExpr::lit(XValue::Integer(3))
            )
        );

        let complex = XExpr::parse("sin(time * 2.0) + cos(time * 3.0)").unwrap();
        assert!(matches!(
            complex,
            XExpr::Binary {
                op: XBinaryOp::Add,
                ..
            }
        ));
    }

    #[test]
    fn test_high_level_api()
    {
        let expr1 = XExpr::lit(1.0f32).add(XExpr::lit(2.0f32));
        assert_eq!(expr1.to_string(), "1.0 + 2.0");

        let expr2 = XExpr::lit(10.0f32).sub(XExpr::lit(5.0f32));
        assert_eq!(expr2.to_string(), "10.0 - 5.0");

        let expr3 = XExpr::lit(3.0f32).mul(XExpr::lit(4.0f32));
        assert_eq!(expr3.to_string(), "3.0 * 4.0");

        let expr4 = XExpr::lit(8.0f32).div(XExpr::lit(2.0f32));
        assert_eq!(expr4.to_string(), "8.0 / 2.0");
    }

    #[test]
    fn test_parse_errors()
    {
        assert!(matches!(
            XExpr::parse("").unwrap_err(),
            XParseError::UnexpectedEndOfInput
        ));

        assert!(matches!(
            XExpr::parse("1 +").unwrap_err(),
            XParseError::UnexpectedEndOfInput
        ));

        assert!(matches!(
            XExpr::parse("sin(").unwrap_err(),
            XParseError::UnexpectedEndOfInput
        ));

        assert!(matches!(
            XExpr::parse("unknown_func(x)").unwrap_err(),
            XParseError::UnknownFunction(_)
        ));

        assert!(matches!(
            XExpr::parse("(1 + 2").unwrap_err(),
            XParseError::UnmatchedParenthesis
        ));

        assert!(matches!(
            XExpr::parse("1.2.3").unwrap_err(),
            XParseError::UnexpectedToken(_)
        ));
    }

    #[test]
    fn test_all_any_functions()
    {
        assert_eq!(
            XExpr::parse("all(vec3(1.0, 1.0, 1.0))").unwrap(),
            XExpr::unary(XUnaryOp::All, XExpr::lit(XValue::vec3(1.0, 1.0, 1.0)))
        );

        assert_eq!(
            XExpr::parse("any(flags)").unwrap(),
            XExpr::unary(XUnaryOp::Any, XExpr::attr("flags"))
        );
    }

    #[test]
    fn test_whitespace_handling()
    {
        assert_eq!(
            XExpr::parse("  1  +  2  ").unwrap(),
            XExpr::parse("1+2").unwrap()
        );

        assert_eq!(
            XExpr::parse("sin( time )").unwrap(),
            XExpr::parse("sin(time)").unwrap()
        );

        assert_eq!(
            XExpr::parse("vec2( 1.0 , 2.0 )").unwrap(),
            XExpr::parse("vec2(1.0,2.0)").unwrap()
        );
    }

    #[test]
    fn test_expression_display()
    {
        let expr = XExpr::sin(XExpr::attr("time"));
        assert_eq!(expr.to_string(), "sin(attr(time))");

        let expr = XExpr::binary(
            XExpr::lit(XValue::vec2(1.0, 0.0)),
            XBinaryOp::Dot,
            XExpr::lit(XValue::vec2(0.0, 1.0)),
        );
        assert_eq!(expr.to_string(), "dot(vec2(1.0, 0.0), vec2(0.0, 1.0))");
    }

    #[test]
    fn test_from_conversions()
    {
        let _val1: XValue = 1.0f32.into();
        let _val2: XValue = 42i32.into();
        let _val3: XValue = (1.0f32, 2.0f32).into();
        let _val4: XValue = (1.0f32, 2.0f32, 3.0f32).into();
    }

    #[test]
    fn test_box_conversions()
    {
        let expr = XExpr::lit(1.0f32);
        let _boxed: Box<XExpr> = expr.into();
    }

    #[test]
    fn test_type_inference_literals()
    {
        let attributes = HashMap::new();
        let props = HashMap::new();

        assert_eq!(
            XExpr::lit(XValue::Float(1.0)).get_result_type(&attributes, &props),
            Some(XExprReturnType::Float)
        );

        assert_eq!(
            XExpr::lit(XValue::Integer(42)).get_result_type(&attributes, &props),
            Some(XExprReturnType::Integer)
        );

        assert_eq!(
            XExpr::lit(XValue::Vec2(1.0, 2.0)).get_result_type(&attributes, &props),
            Some(XExprReturnType::Vec2)
        );

        assert_eq!(
            XExpr::lit(XValue::Vec3(1.0, 2.0, 3.0)).get_result_type(&attributes, &props),
            Some(XExprReturnType::Vec3)
        );
    }

    #[test]
    fn test_type_inference_attributes_and_props()
    {
        let mut attributes = HashMap::new();
        attributes.insert("position".to_string(), XExprReturnType::Vec3);
        attributes.insert("velocity".to_string(), XExprReturnType::Vec2);

        let mut props = HashMap::new();
        props.insert("mass".to_string(), XExprReturnType::Float);
        props.insert("id".to_string(), XExprReturnType::Integer);

        assert_eq!(
            XExpr::attr("position").get_result_type(&attributes, &props),
            Some(XExprReturnType::Vec3)
        );

        assert_eq!(
            XExpr::prop("mass").get_result_type(&attributes, &props),
            Some(XExprReturnType::Float)
        );

        assert_eq!(
            XExpr::attr("unknown").get_result_type(&attributes, &props),
            None
        );
    }

    #[test]
    fn test_type_inference_builtins()
    {
        let attributes = HashMap::new();
        let props = HashMap::new();

        assert_eq!(
            XExpr::builtin(XBuiltInOp::Time).get_result_type(&attributes, &props),
            Some(XExprReturnType::Float)
        );

        assert_eq!(
            XExpr::builtin(XBuiltInOp::Rand).get_result_type(&attributes, &props),
            Some(XExprReturnType::Float)
        );
    }

    #[test]
    fn test_type_inference_unary_operations()
    {
        let attributes = HashMap::new();
        let props = HashMap::new();

        assert_eq!(
            XExpr::sin(XExpr::lit(XValue::Vec2(1.0, 2.0))).get_result_type(&attributes, &props),
            Some(XExprReturnType::Vec2)
        );

        assert_eq!(
            XExpr::abs(XExpr::lit(XValue::Float(-1.0))).get_result_type(&attributes, &props),
            Some(XExprReturnType::Float)
        );

        assert_eq!(
            XExpr::norm(XExpr::lit(XValue::Vec3(1.0, 2.0, 3.0)))
                .get_result_type(&attributes, &props),
            Some(XExprReturnType::Float)
        );

        assert_eq!(
            XExpr::unary(XUnaryOp::All, XExpr::lit(XValue::Vec2(1.0, 0.0)))
                .get_result_type(&attributes, &props),
            Some(XExprReturnType::Float)
        );

        assert_eq!(
            XExpr::unary(XUnaryOp::All, XExpr::lit(XValue::Float(1.0)))
                .get_result_type(&attributes, &props),
            Some(XExprReturnType::Error)
        );
    }

    #[test]
    fn test_type_inference_binary_operations()
    {
        let attributes = HashMap::new();
        let props = HashMap::new();

        assert_eq!(
            XExpr::add(
                XExpr::lit(XValue::Float(1.0)),
                XExpr::lit(XValue::Float(2.0))
            )
            .get_result_type(&attributes, &props),
            Some(XExprReturnType::Float)
        );

        assert_eq!(
            XExpr::add(
                XExpr::lit(XValue::Vec2(1.0, 2.0)),
                XExpr::lit(XValue::Vec2(3.0, 4.0))
            )
            .get_result_type(&attributes, &props),
            Some(XExprReturnType::Vec2)
        );

        assert_eq!(
            XExpr::mul(
                XExpr::lit(XValue::Float(2.0)),
                XExpr::lit(XValue::Vec3(1.0, 2.0, 3.0))
            )
            .get_result_type(&attributes, &props),
            Some(XExprReturnType::Vec3)
        );

        assert_eq!(
            XExpr::mul(
                XExpr::lit(XValue::Vec2(1.0, 2.0)),
                XExpr::lit(XValue::Vec3(1.0, 2.0, 3.0))
            )
            .get_result_type(&attributes, &props),
            Some(XExprReturnType::Error)
        );

        assert_eq!(
            XExpr::binary(
                XExpr::lit(XValue::Vec2(1.0, 2.0)),
                XBinaryOp::Dot,
                XExpr::lit(XValue::Vec2(3.0, 4.0))
            )
            .get_result_type(&attributes, &props),
            Some(XExprReturnType::Float)
        );

        assert_eq!(
            XExpr::binary(
                XExpr::lit(XValue::Vec3(1.0, 2.0, 3.0)),
                XBinaryOp::Cross,
                XExpr::lit(XValue::Vec3(4.0, 5.0, 6.0))
            )
            .get_result_type(&attributes, &props),
            Some(XExprReturnType::Vec3)
        );

        assert_eq!(
            XExpr::binary(
                XExpr::lit(XValue::Vec2(1.0, 2.0)),
                XBinaryOp::Cross,
                XExpr::lit(XValue::Vec2(3.0, 4.0))
            )
            .get_result_type(&attributes, &props),
            Some(XExprReturnType::Error)
        );

        assert_eq!(
            XExpr::binary(
                XExpr::lit(XValue::Float(1.0)),
                XBinaryOp::Lt,
                XExpr::lit(XValue::Float(2.0))
            )
            .get_result_type(&attributes, &props),
            Some(XExprReturnType::Float)
        );
    }

    #[test]
    fn test_type_inference_complex_expressions()
    {
        let mut attributes = HashMap::new();
        attributes.insert("pos".to_string(), XExprReturnType::Vec3);
        attributes.insert("vel".to_string(), XExprReturnType::Vec2);

        let mut props = HashMap::new();
        props.insert("scale".to_string(), XExprReturnType::Float);

        let expr = XExpr::mul(XExpr::norm(XExpr::attr("pos")), XExpr::prop("scale"));
        assert_eq!(
            expr.get_result_type(&attributes, &props),
            Some(XExprReturnType::Float)
        );

        let expr = XExpr::sin(XExpr::attr("vel"));
        assert_eq!(
            expr.get_result_type(&attributes, &props),
            Some(XExprReturnType::Vec2)
        );

        let expr = XExpr::add(XExpr::attr("pos"), XExpr::attr("vel"));
        assert_eq!(
            expr.get_result_type(&attributes, &props),
            Some(XExprReturnType::Error)
        );
    }

    #[test]
    fn test_type_inference_missing_attributes()
    {
        let attributes = HashMap::new();
        let props = HashMap::new();

        assert_eq!(
            XExpr::attr("unknown").get_result_type(&attributes, &props),
            None
        );

        let expr = XExpr::add(XExpr::lit(XValue::Float(1.0)), XExpr::attr("unknown"));
        assert_eq!(expr.get_result_type(&attributes, &props), None);
    }

    #[test]
    fn test_type_inference_workflow_example()
    {
        let mut attributes = HashMap::new();
        attributes.insert("position".to_string(), XExprReturnType::Vec3);
        attributes.insert("velocity".to_string(), XExprReturnType::Vec2);
        attributes.insert("age".to_string(), XExprReturnType::Float);

        let mut props = HashMap::new();
        props.insert("gravity".to_string(), XExprReturnType::Float);
        props.insert("wind_force".to_string(), XExprReturnType::Vec2);

        let examples = vec![
            ("5.0", Some(XExprReturnType::Float)),
            ("42", Some(XExprReturnType::Integer)),
            ("vec2(1.0, 2.0)", Some(XExprReturnType::Vec2)),
            ("vec3(1.0, 2.0, 3.0)", Some(XExprReturnType::Vec3)),
            ("attr(position)", Some(XExprReturnType::Vec3)),
            ("prop(gravity)", Some(XExprReturnType::Float)),
            ("time", Some(XExprReturnType::Float)),
            ("rand", Some(XExprReturnType::Float)),
            ("sin(attr(age))", Some(XExprReturnType::Float)),
            ("norm(attr(position))", Some(XExprReturnType::Float)),
            (
                "attr(velocity) + prop(wind_force)",
                Some(XExprReturnType::Vec2),
            ),
            ("attr(age) * prop(gravity)", Some(XExprReturnType::Float)),
            (
                "dot(attr(position), vec3(0.0, 1.0, 0.0))",
                Some(XExprReturnType::Float),
            ),
            (
                "attr(position) + attr(velocity)",
                Some(XExprReturnType::Error),
            ),
            ("attr(unknown)", None),
        ];

        for (expr_str, expected) in examples {
            let expr = XExpr::parse(expr_str).unwrap();
            let inferred_type = expr.get_result_type(&attributes, &props);
            assert_eq!(
                inferred_type, expected,
                "Failed for expression: {}",
                expr_str
            );
        }
    }

    #[test]
    fn test_parse_complex_vector_expression()
    {
        let expr_str = "(vec3(-0.0, -15.0, -0.0) - (rand * vec3(0.0, 15.0, 0.0)))";
        let parsed = XExpr::parse(expr_str).unwrap();

        match parsed {
            XExpr::Binary {
                left,
                op: XBinaryOp::Sub,
                right,
            } => {
                match *left {
                    XExpr::Lit(XValue::Vec3(x, y, z)) => {
                        assert_eq!(x, -0.0);
                        assert_eq!(y, -15.0);
                        assert_eq!(z, -0.0);
                    }
                    _ => panic!("Expected left side to be a vec3 literal with negative values"),
                }

                match *right {
                    XExpr::Binary {
                        left: rand_expr,
                        op: XBinaryOp::Mul,
                        right: vec_expr,
                    } => {
                        assert!(matches!(*rand_expr, XExpr::BuiltIn(XBuiltInOp::Rand)));
                        assert_eq!(*vec_expr, XExpr::lit(XValue::vec3(0.0, 15.0, 0.0)));
                    }
                    _ => panic!("Expected right side to be multiplication of rand and vec3"),
                }
            }
            _ => panic!("Expected binary subtraction expression"),
        }
    }

    #[test]
    fn test_expression_formatting_without_redundant_parentheses()
    {
        let expr = XExpr::add(
            XExpr::lit(1.0),
            XExpr::mul(
                XExpr::builtin(XBuiltInOp::Rand),
                XExpr::sin(XExpr::builtin(XBuiltInOp::Time)),
            ),
        );

        // Should be "1.0 + rand * sin(time)" not "(1.0 + rand * sin(time))"
        assert_eq!(expr.to_string(), "1.0 + rand * sin(time)");
        let nested_expr = XExpr::mul(
            XExpr::add(XExpr::lit(1.0), XExpr::lit(2.0)),
            XExpr::lit(3.0),
        );

        assert_eq!(nested_expr.to_string(), "(1.0 + 2.0) * 3.0");

        let dot_expr = XExpr::binary(
            XExpr::lit(XValue::vec3(1.0, 2.0, 3.0)),
            XBinaryOp::Dot,
            XExpr::lit(XValue::vec3(4.0, 5.0, 6.0)),
        );
        assert_eq!(
            dot_expr.to_string(),
            "dot(vec3(1.0, 2.0, 3.0), vec3(4.0, 5.0, 6.0))"
        );
    }
}
