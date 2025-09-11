struct Shapes {
    data: array<f32>,
}
@group(0) @binding(0) var<storage, read> shapes: Shapes;

@vertex
fn vertex(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2<f32>(f32(vertex_index >> 1u), f32(vertex_index & 1u)) * 2.0;
    let clip_position = vec4<f32>(uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
    return clip_position;
}

fn sdf_coverage(sd: f32, feather: f32) -> f32 {
    let half  = max(0.5 * feather, 0.5 * fwidth(sd));
    return 1.0 - smoothstep(-half, half, sd);
}

fn sdf_circle(
    frag_pos: vec2<f32>,
    center: vec2<f32>,
    radius: f32,
) -> f32 {
    return length(frag_pos - center) - radius;
}

fn sdf_ring(
    frag_pos: vec2<f32>,
    center: vec2<f32>,
    radius_inner: f32,
    radius_outer: f32
) -> f32 {
    let rin = min(radius_inner, radius_outer);
    let rout = max(radius_inner, radius_outer);
    let d = length(frag_pos - center);
    return abs(d - 0.5 * (rin + rout)) - 0.5 * (rout - rin);
}

fn sdf_box(
    frag_pos: vec2<f32>,
    center: vec2<f32>,
    half: vec2<f32>,
) -> f32 {
    let distance = abs(frag_pos - center) - half;
    return length(max(distance, vec2<f32>(0.0))) + min(max(distance.x, distance.y), 0.0);
}

fn sdf_round_box(
    frag_pos: vec2<f32>,
    center: vec2<f32>,
    half: vec2<f32>,
    radius: f32,
) -> f32 {
    return sdf_box(frag_pos, center, half) - radius;
}

fn rotate(
    frag_pos: vec2<f32>,
    angle: f32,
) -> vec2<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec2<f32>(
        c * frag_pos.x - s * frag_pos.y,
        s * frag_pos.x + c * frag_pos.y,
    );
}

fn sdf_rectangle(
    frag_pos: vec2<f32>,
    center: vec2<f32>,
    half: vec2<f32>,
    radius: f32,  // Percentage of the shorter side.
    angle: f32,  // In degrees
) -> f32 {
    // Rotate into local rect space.
    let local = rotate(frag_pos - center, -angle);

    // Clamp the radius so it can't exceed the shortest half-extent.
    let r = min(radius, min(half.x, half.y));

    let k = half - vec2<f32>(r);
    let q = abs(local) - k;

    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

var<private> shape_index: u32;
var<private> acc_rgb: vec3<f32>;  // Accumulator RGB.
var<private> acc_a: f32;          // Accumulator alpha.

fn command_circle(frag_pos: vec2<f32>) {
    let color = vec4<f32>(
        shapes.data[shape_index + 0],
        shapes.data[shape_index + 1],
        shapes.data[shape_index + 2],
        shapes.data[shape_index + 3],
    );
    let center = vec2<f32>(
        shapes.data[shape_index + 4],
        shapes.data[shape_index + 5],
    );
    let radius = shapes.data[shape_index + 6];
    let feather = shapes.data[shape_index + 7];

    shape_index += 8;

    let sd = sdf_circle(frag_pos, center, radius);
    let coverage = sdf_coverage(sd, feather);
    let a = clamp(color.a * coverage, 0.0, 1.0);
    let pre = color.rgb * a;

    acc_rgb = pre + acc_rgb * (1.0 - a);
    acc_a = a + acc_a * (1.0 - a);
}

fn command_ring(frag_pos: vec2<f32>) {
    let color = vec4<f32>(
        shapes.data[shape_index + 0],
        shapes.data[shape_index + 1],
        shapes.data[shape_index + 2],
        shapes.data[shape_index + 3],
    );
    let center = vec2<f32>(
        shapes.data[shape_index + 4],
        shapes.data[shape_index + 5],
    );
    let radius_inner = shapes.data[shape_index + 6];
    let radius_outer = shapes.data[shape_index + 7];
    let feather = shapes.data[shape_index + 8];

    shape_index += 9;

    let sd = sdf_ring(frag_pos, center, radius_inner, radius_outer);
    let coverage = sdf_coverage(sd, feather);
    let a = clamp(color.a * coverage, 0.0, 1.0);
    let pre = color.rgb * a;

    acc_rgb = pre + acc_rgb * (1.0 - a);
    acc_a = a + acc_a * (1.0 - a);
}

fn command_rectangle(frag_pos: vec2<f32>) {
    let color = vec4<f32>(
        shapes.data[shape_index + 0],
        shapes.data[shape_index + 1],
        shapes.data[shape_index + 2],
        shapes.data[shape_index + 3],
    );
    let center = vec2<f32>(
        shapes.data[shape_index + 4],
        shapes.data[shape_index + 5],
    );
    let half_size = vec2<f32>(
        shapes.data[shape_index + 6],
        shapes.data[shape_index + 7],
    );
    let radius = shapes.data[shape_index + 8];
    let angle = shapes.data[shape_index + 9];
    let feather = shapes.data[shape_index + 10];

    shape_index += 11;

    let sd = sdf_rectangle(
        frag_pos,
        center,
        half_size,
        radius,
        radians(angle),
    );
    let coverage = sdf_coverage(sd, feather);
    let a = clamp(color.a * coverage, 0.0, 1.0);
    let pre = color.rgb * a;

    acc_rgb = pre + acc_rgb * (1.0 - a);
    acc_a = a + acc_a * (1.0 - a);
}

@fragment
fn fragment(@builtin(position) frag_pos4: vec4<f32>) -> @location(0) vec4<f32> {
    // Reset the state.
    shape_index = 0;
    acc_rgb = vec3<f32>(0.0, 0.0, 0.0);
    acc_a = 0.0;

    let frag_pos = frag_pos4.xy;

    loop {
        let id = bitcast<u32>(shapes.data[shape_index]);
        shape_index += 1;

        if id == 0 {
            break;
        }

        switch id {
            case 1: {
                command_circle(frag_pos);
            }

            case 2: {
                command_ring(frag_pos);
            }

            case 3: {
                command_rectangle(frag_pos);
            }

            default: {}
        }
    }

    return vec4<f32>(
        acc_rgb + (1.0 - acc_a),
        1.0
    );
}
