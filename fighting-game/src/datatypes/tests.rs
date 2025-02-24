use super::*;

macro_rules! assert_float_eq {
    ($a:expr, $b:expr) => {
        if !($a - $b < f32::EPSILON || $b - $a < f32::EPSILON) {
            assert_eq!($a, $b);
        }
    };
}

#[test]
fn vector_tests() {
    assert_eq!(Vector2::new(1.0, 3.0), Vector2 { x: 1.0, y: 3.0 });
    assert_eq!(vector!(1.0, 3.0), Vector2 { x: 1.0, y: 3.0 });

    {
        let v = Vector2::new(3.0, -3.0).normalized();
        assert_float_eq!(v.x, 2.0f32.sqrt() / 2.0);
        assert_float_eq!(v.y, -2.0f32.sqrt() / 2.0);
    }

    assert_eq!(Vector2::new(3.0, 4.0).magnitude(), 5.0);

    assert_eq!(
        Vector2::new(4.0, 12.0).to(&Vector2::new(8.0, 2.0)),
        Vector2 { x: 4.0, y: -10.0 }
    );

    assert_eq!(
        Vector2::new(-28.3, 112.52).y(5.8),
        Vector2 { x: -28.3, y: 5.8 }
    );
    assert_eq!(
        Vector2::new(-28.3, 112.52).x(5.8),
        Vector2 { x: 5.8, y: 112.52 }
    );

    assert_eq!(
        Vector2::new(4.0, 1.0) + Vector2::new(2.5, -2.0),
        Vector2 { x: 6.5, y: -1.0 }
    );
    assert_eq!(
        Vector2::new(4.0, 1.0) - Vector2::new(2.5, -2.0),
        Vector2 { x: 1.5, y: 3.0 }
    );

    assert_eq!(Vector2::new(12.0, 8.0) * 3.0, Vector2 { x: 36.0, y: 24.0 });
    assert_eq!(Vector2::new(12.0, 8.0) / 4.0, Vector2 { x: 3.0, y: 2.0 });
}

#[test]
fn bounding_box_test() {
    assert_eq!(
        BoundingBox::new(vector!(1.0, 2.0), vector!(8.0, 23.0)),
        BoundingBox {
            min: vector!(1.0, 2.0),
            max: vector!(8.0, 23.0)
        }
    );
    assert_eq!(
        BoundingBox::new(vector!(8.0, 2.0), vector!(1.0, 23.0)),
        BoundingBox {
            min: vector!(1.0, 2.0),
            max: vector!(8.0, 23.0)
        }
    );

    let pos_size_bb = BoundingBox::pos_size(vector!(5.0, 8.0), vector!(4.0, 3.0));

    assert_eq!(
        pos_size_bb,
        BoundingBox {
            min: vector!(3.0, 6.5),
            max: vector!(7.0, 9.5)
        }
    );
    assert_eq!(pos_size_bb.size(), vector!(4.0, 3.0));
    assert_eq!(pos_size_bb.position(), vector!(5.0, 8.0));

    let moved = pos_size_bb.clone().transformed(vector!(8.0, -2.0));
    assert_eq!(moved.size(), vector!(4.0, 3.0));
    assert_eq!(moved.position(), vector!(13.0, 6.0));

    let mut transformed_by = pos_size_bb.clone();
    transformed_by.transform_by(vector!(8.0, -2.0));
    assert_eq!(transformed_by, moved);

    assert_eq!(BoundingBox::from_point_cloud([].into_iter()), None);
    assert_eq!(
        BoundingBox::from_point_cloud([vector!(12.5, 31.8)].into_iter()),
        Some(BoundingBox {
            min: vector!(12.5, 31.8),
            max: vector!(12.5, 31.8),
        })
    );
    assert_eq!(
        BoundingBox::from_point_cloud(
            [
                vector!(8.0, 5.3),
                vector!(12.5, 3.0),
                vector!(9.42, 8.0),
                vector!(7.3, 9.0),
                vector!(8.5, -3.0)
            ]
            .into_iter()
        ),
        Some(BoundingBox {
            min: vector!(7.3, -3.0),
            max: vector!(12.5, 9.0),
        })
    );

    assert_eq!(
        BoundingBox::new(vector!(8.3, 12.5), vector!(22.0, 15.5)).distance(vector!(21.0, 9.0)),
        vector!(0.0, -3.5)
    );
    assert_eq!(
        BoundingBox::new(vector!(8.3, 12.5), vector!(22.0, 15.5)).distance(vector!(28.0, 9.0)),
        vector!(6.0, -3.5)
    );

    assert_eq!(
        BoundingBox::new(vector!(10.0, 10.0), vector!(15.0, 18.0))
            .overlap(&BoundingBox::new(vector!(8.0, 12.0), vector!(12.0, 16.0))),
        vector!(2.0, 4.0)
    );
}

#[test]
fn bounding_circle_tests() {
    assert_eq!(
        BoundingCircle::new(vector!(4.0, 3.0), 5.0),
        BoundingCircle {
            position: vector!(4.0, 3.0),
            radius: 5.0
        }
    );

    let mut circle = BoundingCircle::new(vector!(4.0, 3.0), 5.0);
    let moved = circle.clone().transformed(vector!(3.0, -1.0));

    assert_eq!(
        moved,
        BoundingCircle {
            position: vector!(7.0, 2.0),
            radius: 5.0
        }
    );

    circle.transform_by(vector!(3.0, -1.0));
    assert_eq!(circle, moved);
}

#[test]
fn intersection_tests() {
    let bounding_box = BoundingBox::new(vector!(12.0, 8.0), vector!(24.0, 18.0));
    assert!(bounding_box.intersects(&vector!(15.8, 12.3)));
    assert!(!bounding_box.intersects(&vector!(10.8, 12.3)));
    assert!(!bounding_box.intersects(&vector!(15.8, 1.3)));
    assert!(BoundingBox::new(vector!(10.0, 10.0), vector!(15.0, 18.0))
        .intersects(&BoundingBox::new(vector!(8.0, 12.0), vector!(12.0, 16.0))));
    assert!(!BoundingBox::new(vector!(10.0, 10.0), vector!(15.0, 18.0))
        .intersects(&BoundingBox::new(vector!(8.0, 12.0), vector!(9.0, 16.0))));

    let bounding_circle = BoundingCircle::new(vector!(10.0, 12.0), 5.0);
    assert!(bounding_circle.intersects(&vector!(8.0, 12.0)));
    assert!(!bounding_circle.intersects(&vector!(4.0, 12.0)));
    assert!(!bounding_circle.intersects(&vector!(8.0, 2.0)));
    assert!(bounding_circle.intersects(&BoundingCircle::new(vector!(6.0, 8.0), 2.0)));
    assert!(!bounding_circle.intersects(&BoundingCircle::new(vector!(6.0, 4.0), 2.0)));
    assert!(bounding_circle.intersects(&bounding_box));
    assert!(bounding_box.intersects(&bounding_circle));

    assert!(bounding_circle.intersects(&bounding_circle));
    assert!(bounding_box.intersects(&bounding_box));

    let tangent_circle = BoundingCircle::new(vector!(5.0, 5.0), 3.0);
    assert!(!tangent_circle.intersects(&vector!(2.0, 5.0)));
    let tangent_box = BoundingBox::new(vector!(-3.0, -3.0), vector!(12.0, 2.0));
    assert!(!tangent_circle.intersects(&tangent_box));
    assert!(!tangent_box.intersects(&tangent_circle));
    assert!(!tangent_box.intersects(&vector!(8.0, 3.0)));

    let box_a = BoundingBox::pos_size(vector!(2.0, 0.0), vector!(4.0, 6.0));
    let box_b = BoundingBox::pos_size(vector!(6.0, 0.0), vector!(4.0, 6.0));
    assert!(!box_a.intersects(&box_b));
}
