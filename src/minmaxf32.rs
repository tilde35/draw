pub(crate) fn minf(a: f32, b: f32) -> f32 {
    if a < b {
        a
    } else {
        b
    }
}

pub(crate) fn minf3(a: f32, b: f32, c: f32) -> f32 {
    minf(minf(a, b), c)
}

pub(crate) fn maxf(a: f32, b: f32) -> f32 {
    if a > b {
        a
    } else {
        b
    }
}

pub(crate) fn maxf3(a: f32, b: f32, c: f32) -> f32 {
    maxf(maxf(a, b), c)
}

pub(crate) fn valid_range(v: f32, name: &str, min: f32, max: f32) -> f32 {
    if v >= min && v <= max {
        v
    } else {
        panic!(
            "Invalid value for {} {}, must be between {} and {}",
            name, v, min, max
        )
    }
}

pub(crate) fn offset_range(v: f32, offset: f32, name: &str, min: f32, max: f32) -> f32 {
    if !offset.is_finite() {
        panic!("Value must a valid number for {}", name);
    }
    let result = v + offset;
    if result <= min {
        min
    } else if result >= max {
        max
    } else {
        result
    }
}
