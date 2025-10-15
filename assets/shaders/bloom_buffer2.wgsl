fn makeBloom(lod: f32, offset: vec2<f32>, bCoord: vec2<f32>) -> vec3<f32> {
	var offset_var = offset;
	let pixelSize: vec2<f32> = 1. / vec2<f32>(uni.iResolution.x, uni.iResolution.y);
	offset_var = offset_var + (pixelSize);
	let lodFactor: f32 = exp2(lod);
	var bloom: vec3<f32> = vec3<f32>(0.);
	let scale: vec2<f32> = lodFactor * pixelSize;
	let coord: vec2<f32> = (bCoord.xy - offset_var) * lodFactor;
	var totalWeight: f32 = 0.;
	if (any(greaterThanEqual(abs(coord - 0.5), scale + 0.5))) {	return vec3<f32>(0.);
 }

	for (var i: i32 = -5; i < 5; i = i + 1) {

		for (var j: i32 = -5; j < 5; j = j + 1) {
			let wg: f32 = pow(1. - length(vec2<f32>(i, j)) * 0.125, 6.);
			bloom = pow(sample_texture(BUFFER_iChannel0, vec2<f32>(i, j) * scale + lodFactor * pixelSize + coord).rgb, vec3<f32>(2.2)) * wg + bloom;
			totalWeight = totalWeight + (wg);
		}

	}

	bloom = bloom / (totalWeight);
	return bloom;
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let R: vec2<f32> = uni.iResolution.xy;
    let y_inverted_location = vec2<i32>(i32(invocation_id.x), i32(R.y) - i32(invocation_id.y));
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

	var fragColor: vec4<f32>;
	var fragCoord = vec2<f32>(f32(location.x), f32(location.y) );

	let uv: vec2<f32> = fragCoord / uni.iResolution.xy;
	var blur: vec3<f32> = makeBloom(2., vec2<f32>(0., 0.), uv);
	blur = blur + (makeBloom(3., vec2<f32>(0.3, 0.), uv));
	blur = blur + (makeBloom(4., vec2<f32>(0., 0.3), uv));
	blur = blur + (makeBloom(5., vec2<f32>(0.1, 0.3), uv));
	blur = blur + (makeBloom(6., vec2<f32>(0.2, 0.3), uv));
	fragColor = vec4<f32>(pow(blur, vec3<f32>(1. / 2.2)), 1.);
}

