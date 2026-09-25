// Juice (docs/JUICE.md) — bloom on emissives only. The layers that glow (torch flames, fire and gas, embers and sparks, bolts)
// write the target's EMISSIVE tag (a = 0.625, treated as a sprite by the blit); this pass keeps only those pixels over a luma
// knee, at half the target's size, blurs them with a separable 9-tap Gaussian (two ping-pong passes), and the blit adds the result
// (linear-filtered: a smooth halo under a crisp pixel image). Three passes over ≤ 1/4 of the target's pixels: ~0.1 ms.
import * as THREE from "three";

export const EMISSIVE_TAG = 0.625;

const VERT = /* glsl */ `
varying vec2 vUv;
void main() { vUv = uv; gl_Position = vec4(position.xy, 0.0, 1.0); }`;

const BRIGHT = /* glsl */ `
uniform sampler2D tex;
uniform vec2 uTexel;   // one SOURCE texel
varying vec2 vUv;
vec3 pick(vec2 uv) {
  vec4 s = texture2D(tex, uv);
  float em = 1.0 - step(0.03, abs(s.a - ${EMISSIVE_TAG}));
  float m = max(s.r, max(s.g, s.b));
  // hot colours bloom (flame, ember, spark, soul); the gas's sickly green only a little
  float green = clamp((s.g - max(s.r, s.b)) * 4.0, 0.0, 0.75);
  return s.rgb * em * smoothstep(0.6, 0.95, m) * (1.0 - green);
}
void main() {
  // 2×2 box down to half size (every source texel counted: a one-texel spark still blooms)
  vec3 c = pick(vUv + uTexel * vec2(-0.5, -0.5)) + pick(vUv + uTexel * vec2(0.5, -0.5)) + pick(vUv + uTexel * vec2(-0.5, 0.5)) + pick(vUv + uTexel * vec2(0.5, 0.5));
  gl_FragColor = vec4(c * 0.25, 1.0);
}`;

const BLUR = /* glsl */ `
uniform sampler2D tex;
uniform vec2 uDir;     // one texel along the blur axis
varying vec2 vUv;
void main() {
  vec3 c = texture2D(tex, vUv).rgb * 0.227;
  c += (texture2D(tex, vUv + uDir * 1.385).rgb + texture2D(tex, vUv - uDir * 1.385).rgb) * 0.316;
  c += (texture2D(tex, vUv + uDir * 3.231).rgb + texture2D(tex, vUv - uDir * 3.231).rgb) * 0.070;
  gl_FragColor = vec4(c, 1.0);
}`;

export class Bloom {
  readonly a: THREE.WebGLRenderTarget;
  private b: THREE.WebGLRenderTarget;
  private scene = new THREE.Scene();
  private camera = new THREE.OrthographicCamera(-1, 1, 1, -1, 0, 1);
  private quad: THREE.Mesh;
  private bright: THREE.ShaderMaterial;
  private blur: THREE.ShaderMaterial;
  private w = 1; private h = 1;

  constructor(source: THREE.Texture) {
    const opt = { minFilter: THREE.LinearFilter, magFilter: THREE.LinearFilter, generateMipmaps: false, depthBuffer: false, stencilBuffer: false, colorSpace: THREE.NoColorSpace };
    this.a = new THREE.WebGLRenderTarget(2, 2, opt);
    this.b = new THREE.WebGLRenderTarget(2, 2, opt);
    this.bright = new THREE.ShaderMaterial({ vertexShader: VERT, fragmentShader: BRIGHT, depthTest: false, depthWrite: false,
      uniforms: { tex: { value: source }, uTexel: { value: new THREE.Vector2() } } });
    this.blur = new THREE.ShaderMaterial({ vertexShader: VERT, fragmentShader: BLUR, depthTest: false, depthWrite: false,
      uniforms: { tex: { value: null }, uDir: { value: new THREE.Vector2() } } });
    this.quad = new THREE.Mesh(new THREE.PlaneGeometry(2, 2), this.bright);
    this.quad.frustumCulled = false;
    this.scene.add(this.quad);
  }

  /** source size in texels (the pixel target) */
  setSize(sw: number, sh: number): void {
    const w = Math.max(1, sw >> 1), h = Math.max(1, sh >> 1);
    (this.bright.uniforms.uTexel!.value as THREE.Vector2).set(1 / sw, 1 / sh);
    if (w === this.w && h === this.h) return;
    this.w = w; this.h = h;
    this.a.setSize(w, h); this.b.setSize(w, h);
  }

  render(renderer: THREE.WebGLRenderer): void {
    this.quad.material = this.bright;
    renderer.setRenderTarget(this.a); renderer.render(this.scene, this.camera);
    this.quad.material = this.blur;
    const u = this.blur.uniforms, d = u.uDir!.value as THREE.Vector2;
    u.tex!.value = this.a.texture; d.set(1 / this.w, 0);
    renderer.setRenderTarget(this.b); renderer.render(this.scene, this.camera);
    u.tex!.value = this.b.texture; d.set(0, 1 / this.h);
    renderer.setRenderTarget(this.a); renderer.render(this.scene, this.camera);
  }

  dispose(): void { this.a.dispose(); this.b.dispose(); this.bright.dispose(); this.blur.dispose(); }
}
