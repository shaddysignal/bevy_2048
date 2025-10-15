fn getSquare(p: vec2<f32>, rp: vec2<f32>) -> f32 {
	var p_var = p;
	p_var = p_var * (vec2<f32>(uni.iResolution.x, uni.iResolution.y));
	p_var = p_var / (max(uni.iResolution.x, uni.iResolution.y));
	p_var = p_var + (rp);
	let bl: vec2<f32> = step(abs(p_var * 2. - 1.), vec2<f32>(0.2));
	let rt: f32 = bl.x * bl.y;
	return rt;
}

fn getCircle(p: vec2<f32>, rp: vec2<f32>) -> f32 {
	var p_var = p;
	p_var = p_var * (vec2<f32>(uni.iResolution.x, uni.iResolution.y));
	p_var = p_var / (max(uni.iResolution.x, uni.iResolution.y));
	return step(distance(p_var, rp), 0.1);
}

fn getTriangle(p: vec2<f32>, rp: vec2<f32>) -> f32 {
	var p_var = p;
	p_var = p_var * (vec2<f32>(uni.iResolution.x, uni.iResolution.y));
	p_var = p_var / (max(uni.iResolution.x, uni.iResolution.y));
	p_var = p_var - (rp);
	var color: vec3<f32> = vec3<f32>(0.);
	var d: f32 = 0.;
	p_var = p_var * 2. - 1.;
	let N: i32 = 3;
	let a: f32 = atan(p_var.x, p_var.y) + 3.1415927;
	let r: f32 = 6.2831855 / f32(N);
	d = cos(floor(0.5 + a / r) * r - a) * length(p_var);
	return 1. - step(0.12, d);
}

fn getTexture(uv: vec2<f32>) -> vec3<f32> {
	let textureSample: vec4<f32> = sample_texture(BUFFER_iChannel0, uv);
	return sqrt(textureSample.rgb * textureSample.a);
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let R: vec2<f32> = uni.iResolution.xy;
    let y_inverted_location = vec2<i32>(i32(invocation_id.x), i32(R.y) - i32(invocation_id.y));
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

	var fragColor: vec4<f32>;
	var fragCoord = vec2<f32>(f32(location.x), f32(location.y) );

	let uv: vec2<f32> = fragCoord / vec2<f32>(uni.iResolution.x, uni.iResolution.y);
	let rectangle: vec3<f32> = getSquare(uv, vec2<f32>(-0.3, 0.21)) * vec3<f32>(2., 12., 30.) * 2.;
	let circle: vec3<f32> = getCircle(uv, vec2<f32>(0.2, 0.29)) * vec3<f32>(30., 12., 2.) * 2.;
	let triangle_: vec3<f32> = getTriangle(uv, vec2<f32>(0., -0.23)) * vec3<f32>(2., 30., 2.) * 2.;
	let color: vec3<f32> = rectangle + circle + triangle_;
	fragColor = vec4<f32>(pow(color, vec3<f32>(1. / 2.2)) / 24., 1.);
}

