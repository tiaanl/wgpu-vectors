struct Draw {
    min: vec2<f32>,
    max: vec2<f32>,
    op_code_index: u32,
}

struct Draws {
    draws: array<Draw>,
}

struct OpCodes {
    op_codes: array<f32>,
}

@group(0) @binding(0)
var<storage, read> draws: Draws;

@group(0) @binding(1)
var<storage, read> op_codes: OpCodes;

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

fn sdf_ellipse(
    frag_pos: vec2<f32>,
    center: vec2<f32>,
    radius: vec2<f32>,
) -> f32 {
    return length(frag_pos - center) - radius.x;
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
    angle: f32,
) -> f32 {
    // Rotate into local rect space.
    let local = rotate(frag_pos - center, -angle);

    // Clamp the radius so it can't exceed the shortest half-extent.
    let r = min(radius, min(half.x, half.y));

    let k = half - vec2<f32>(r);
    let q = abs(local) - k;

    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

var<private> op_code_index: u32;
var<private> acc_rgb: vec3<f32>;  // Accumulator RGB.
var<private> acc_a: f32;          // Accumulator alpha.

struct Fill {
    color: vec4<f32>,
    feather: f32,
}

fn read_fill() -> Fill {
    let color = vec4<f32>(
        op_codes.op_codes[op_code_index + 0],
        op_codes.op_codes[op_code_index + 1],
        op_codes.op_codes[op_code_index + 2],
        op_codes.op_codes[op_code_index + 3],
    );
    let feather = op_codes.op_codes[op_code_index + 4];

    op_code_index += 5;

    return Fill(color, feather);
}

struct Stroke {
    color: vec4<f32>,
    thickness: f32,
    feather: f32,
}

fn read_stroke() -> Stroke {
    let color = vec4<f32>(
        op_codes.op_codes[op_code_index + 0],
        op_codes.op_codes[op_code_index + 1],
        op_codes.op_codes[op_code_index + 2],
        op_codes.op_codes[op_code_index + 3],
    );
    let thickness = op_codes.op_codes[op_code_index + 4];
    let feather = op_codes.op_codes[op_code_index + 5];

    op_code_index += 6;

    return Stroke(color, thickness, feather);
}

fn command_ellipse(frag_pos: vec2<f32>) {
    let center = vec2<f32>(
        op_codes.op_codes[op_code_index + 0],
        op_codes.op_codes[op_code_index + 1],
    );
    let radius = vec2<f32>(
        op_codes.op_codes[op_code_index + 2],
        op_codes.op_codes[op_code_index + 3],
    );
    op_code_index += 4;

    let fill = read_fill();
    let stroke = read_stroke();

    let sd = sdf_ellipse(frag_pos, center, radius);
    let coverage = sdf_coverage(sd, fill.feather);
    let a = clamp(fill.color.a * coverage, 0.0, 1.0);
    let pre = fill.color.rgb * a;

    acc_rgb = pre + acc_rgb * (1.0 - a);
    acc_a = a + acc_a * (1.0 - a);
}

fn command_ring(frag_pos: vec2<f32>) {
    let color = vec4<f32>(
        op_codes.op_codes[op_code_index + 0],
        op_codes.op_codes[op_code_index + 1],
        op_codes.op_codes[op_code_index + 2],
        op_codes.op_codes[op_code_index + 3],
    );
    let center = vec2<f32>(
        op_codes.op_codes[op_code_index + 4],
        op_codes.op_codes[op_code_index + 5],
    );
    let radius_inner = op_codes.op_codes[op_code_index + 6];
    let radius_outer = op_codes.op_codes[op_code_index + 7];
    let feather = op_codes.op_codes[op_code_index + 8];

    op_code_index += 9;

    let sd = sdf_ring(frag_pos, center, radius_inner, radius_outer);
    let coverage = sdf_coverage(sd, feather);
    let a = clamp(color.a * coverage, 0.0, 1.0);
    let pre = color.rgb * a;

    acc_rgb = pre + acc_rgb * (1.0 - a);
    acc_a = a + acc_a * (1.0 - a);
}

fn command_rectangle(frag_pos: vec2<f32>) {
    let center = vec2<f32>(
        op_codes.op_codes[op_code_index + 0],
        op_codes.op_codes[op_code_index + 1],
    );
    let extent = vec2<f32>(
        op_codes.op_codes[op_code_index + 2],
        op_codes.op_codes[op_code_index + 3],
    );
    let corner_radius = op_codes.op_codes[op_code_index + 4];

    op_code_index += 5;

    let fill = read_fill();
    let stroke = read_stroke();

    let angle = 0.0;

    let sd = sdf_rectangle(
        frag_pos,
        center,
        extent,
        corner_radius,
        radians(angle),
    );
    let coverage = sdf_coverage(sd, fill.feather);
    let a = clamp(fill.color.a * coverage, 0.0, 1.0);
    let pre = fill.color.rgb * a;

    acc_rgb = pre + acc_rgb * (1.0 - a);
    acc_a = a + acc_a * (1.0 - a);
}

@fragment
fn fragment(@builtin(position) frag_pos4: vec4<f32>) -> @location(0) vec4<f32> {
    // Reset the state.
    op_code_index = 0;
    acc_rgb = vec3<f32>(0.0, 0.0, 0.0);
    acc_a = 0.0;

    let frag_pos = frag_pos4.xy;

    loop {
        let id = bitcast<u32>(op_codes.op_codes[op_code_index]);
        op_code_index += 1;

        if id == 0 {
             break;
        }

        switch id {
            case 1: {
                command_rectangle(frag_pos);
            }

            case 2: {
                command_ellipse(frag_pos);
            }

            default: {
                break;
            }
        }
    }

    return vec4<f32>(
        acc_rgb + (1.0 - acc_a),
        1.0
    );
}
