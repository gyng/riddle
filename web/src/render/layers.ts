// One instanced quad layer = one draw call. Per-instance: world position (bottom-centre anchor),
// size (world units = env texels), atlas UV rect, params (dim, flash, fade, flip).
// Alpha channel of the target is used as a layer tag read by the blit shader: 1.0 = environment
// (palette-quantised + dithered), 0.5 = sprite (50% palette tint, no dither). Blending is off, so
// each fragment's alpha lands in the target verbatim.
import * as THREE from "three";

export const BAYER_GLSL = /* glsl */ `
float b2(vec2 q) { return 2.0 * q.x + 3.0 * q.y - 4.0 * q.x * q.y; }
float bayer4(vec2 p) {
  vec2 q = floor(mod(p, 4.0));
  return (4.0 * b2(mod(q, 2.0)) + b2(floor(q * 0.5))) / 16.0;
}`;

const VERT = /* glsl */ `
attribute vec3 iPos;
attribute vec2 iSize;
attribute vec4 iUV;
attribute vec4 iParam;
varying vec2 vUv;
varying vec4 vParam;
void main() {
  float u = iParam.w > 0.5 ? 1.0 - uv.x : uv.x;
  vUv = vec2(mix(iUV.x, iUV.z, u), mix(iUV.y, iUV.w, uv.y));
  vParam = iParam;
  vec2 p = iPos.xy + vec2((uv.x - 0.5) * iSize.x, uv.y * iSize.y);
  gl_Position = projectionMatrix * modelViewMatrix * vec4(p, iPos.z, 1.0);
}`;

const FRAG = /* glsl */ `
uniform sampler2D map;
uniform float uTag;
uniform vec3 uFlash;
varying vec2 vUv;
varying vec4 vParam;
${BAYER_GLSL}
void main() {
  vec4 c = texture2D(map, vUv);
  if (c.a < 0.5) discard;
  // screen-door dissolve (die fade) on the target pixel grid
  if (vParam.z > 0.0 && vParam.z > bayer4(gl_FragCoord.xy)) discard;
  vec3 col = c.rgb * vParam.x;
  col = mix(col, uFlash, vParam.y);
  gl_FragColor = vec4(col, uTag);
}`;

export class QuadLayer {
  readonly mesh: THREE.Mesh;
  private geom: THREE.InstancedBufferGeometry;
  private pos: THREE.InstancedBufferAttribute;
  private size: THREE.InstancedBufferAttribute;
  private uvr: THREE.InstancedBufferAttribute;
  private par: THREE.InstancedBufferAttribute;
  private n = 0;
  private lastN = 0;
  private dirty = true; // any instance value changed since the last upload
  readonly capacity: number;

  constructor(texture: THREE.Texture, capacity: number, tag: number, renderOrder: number) {
    this.capacity = capacity;
    const base = new THREE.PlaneGeometry(1, 1);
    base.translate(0.5, 0.5, 0); // uv (0,0) at bottom-left; shader re-anchors to bottom-centre
    const g = new THREE.InstancedBufferGeometry();
    g.index = base.index;
    g.setAttribute("position", base.getAttribute("position"));
    g.setAttribute("uv", base.getAttribute("uv"));
    this.pos = new THREE.InstancedBufferAttribute(new Float32Array(capacity * 3), 3);
    this.size = new THREE.InstancedBufferAttribute(new Float32Array(capacity * 2), 2);
    this.uvr = new THREE.InstancedBufferAttribute(new Float32Array(capacity * 4), 4);
    this.par = new THREE.InstancedBufferAttribute(new Float32Array(capacity * 4), 4);
    for (const a of [this.pos, this.size, this.uvr, this.par]) a.setUsage(THREE.DynamicDrawUsage);
    g.setAttribute("iPos", this.pos);
    g.setAttribute("iSize", this.size);
    g.setAttribute("iUV", this.uvr);
    g.setAttribute("iParam", this.par);
    g.instanceCount = 0;
    this.geom = g;
    const mat = new THREE.ShaderMaterial({
      vertexShader: VERT,
      fragmentShader: FRAG,
      uniforms: { map: { value: texture }, uTag: { value: tag }, uFlash: { value: new THREE.Color(0.98, 0.95, 0.9) } },
      transparent: false,
      depthTest: true,
      depthWrite: true,
      side: THREE.DoubleSide,
    });
    this.mesh = new THREE.Mesh(g, mat);
    this.mesh.frustumCulled = false;
    this.mesh.renderOrder = renderOrder;
  }

  begin(): void { this.n = 0; }

  // Compare-and-write: the GPU upload in end() is skipped when nothing changed (static floors,
  // paused replay), which is most frames for the tile/item layers.
  push(x: number, y: number, z: number, w: number, h: number,
       u0: number, v0: number, u1: number, v1: number,
       dim = 1, flash = 0, fade = 0, flip = 0): void {
    if (this.n >= this.capacity) return;
    const i = this.n++;
    const P = this.pos.array as Float32Array, S = this.size.array as Float32Array;
    const U = this.uvr.array as Float32Array, R = this.par.array as Float32Array;
    const p3 = i * 3, s2 = i * 2, q4 = i * 4;
    if (P[p3] !== x || P[p3 + 1] !== y || P[p3 + 2] !== z) { P[p3] = x; P[p3 + 1] = y; P[p3 + 2] = z; this.dirty = true; }
    if (S[s2] !== w || S[s2 + 1] !== h) { S[s2] = w; S[s2 + 1] = h; this.dirty = true; }
    if (U[q4] !== u0 || U[q4 + 1] !== v0 || U[q4 + 2] !== u1 || U[q4 + 3] !== v1) { U[q4] = u0; U[q4 + 1] = v0; U[q4 + 2] = u1; U[q4 + 3] = v1; this.dirty = true; }
    if (R[q4] !== dim || R[q4 + 1] !== flash || R[q4 + 2] !== fade || R[q4 + 3] !== flip) { R[q4] = dim; R[q4 + 1] = flash; R[q4 + 2] = fade; R[q4 + 3] = flip; this.dirty = true; }
  }

  end(): void {
    this.geom.instanceCount = this.n;
    this.mesh.visible = this.n > 0;
    if (!this.dirty && this.n <= this.lastN) { this.lastN = this.n; return; }
    for (const a of [this.pos, this.size, this.uvr, this.par]) { a.needsUpdate = true; a.clearUpdateRanges(); a.addUpdateRange(0, this.n * a.itemSize); }
    this.lastN = this.n;
    this.dirty = false;
  }

  get count(): number { return this.n; }

  dispose(): void { this.geom.dispose(); (this.mesh.material as THREE.Material).dispose(); }
}
