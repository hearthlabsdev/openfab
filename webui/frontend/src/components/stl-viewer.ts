import { LitElement, html, css } from 'lit';
import { customElement, property } from 'lit/decorators.js';
import * as THREE from 'three';
import { STLLoader } from 'three/examples/jsm/loaders/STLLoader.js';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';

@customElement('stl-viewer')
export class StlViewer extends LitElement {
  @property({ type: String }) src = '';
  @property({ type: String }) color = '#0055ff';

  static styles = css`
    :host {
      display: block;
      width: 100%;
      height: 400px;
      position: relative;
      background: #1a1a1a;
      overflow: hidden;
      border-radius: 8px;
    }
    canvas {
      width: 100%;
      height: 100%;
      display: block;
    }
    .loading {
      position: absolute;
      top: 50%;
      left: 50%;
      transform: translate(-50%, -50%);
      color: #fff;
      font-family: sans-serif;
      font-size: 14px;
      pointer-events: none;
    }
  `;

  private containerRef: HTMLElement | null = null;
  private scene!: THREE.Scene;
  private camera!: THREE.PerspectiveCamera;
  private renderer!: THREE.WebGLRenderer;
  private controls!: OrbitControls;
  private animationFrameId: number | null = null;
  private currentMesh: THREE.Mesh | null = null;
  private resizeObserver!: ResizeObserver;

  firstUpdated() {
    this.containerRef = this.shadowRoot?.querySelector('#canvas-container') ?? null;
    this.initThree();
    if (this.src) {
      this.loadModel(this.src);
    }
  }

  updated(changedProperties: Map<string, any>) {
    if (changedProperties.has('src') && this.src) {
      this.loadModel(this.src);
    }
    if (changedProperties.has('color') && this.currentMesh) {
      (this.currentMesh.material as THREE.MeshStandardMaterial).color.set(this.color);
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    // Clean up Three.js resources to prevent memory leaks
    if (this.animationFrameId !== null) {
      cancelAnimationFrame(this.animationFrameId);
    }
    this.resizeObserver?.disconnect();
    this.renderer?.dispose();
    this.controls?.dispose();
  }

  private initThree() {
    if (!this.containerRef) return;

    const width = this.containerRef.clientWidth || 300;
    const height = this.containerRef.clientHeight || 300;

    // 1. Scene & Camera
    this.scene = new THREE.Scene();
    this.camera = new THREE.PerspectiveCamera(45, width / height, 0.1, 1000);
    this.camera.position.set(100, 100, 100);

    // 2. Renderer
    this.renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
    this.renderer.setSize(width, height);
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    this.containerRef.appendChild(this.renderer.domElement);

    // 3. Controls
    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.controls.enableDamping = true;
    this.controls.dampingFactor = 0.05;

    // 4. Lighting (Crucial for 3D depth perception)
    const ambientLight = new THREE.AmbientLight(0xffffff, 0.6);
    this.scene.add(ambientLight);

    const dirLight1 = new THREE.DirectionalLight(0xffffff, 0.8);
    dirLight1.position.set(1, 2, 3);
    this.scene.add(dirLight1);

    const dirLight2 = new THREE.DirectionalLight(0xffffff, 0.4);
    dirLight2.position.set(-1, -2, -1);
    this.scene.add(dirLight2);

    // 5. Responsive Resize Observer (handles layout shifts smoothly in Lit)
    this.resizeObserver = new ResizeObserver((entries) => {
      for (let entry of entries) {
        const { width, height } = entry.contentRect;
        if (width > 0 && height > 0) {
          this.camera.aspect = width / height;
          this.camera.updateProjectionMatrix();
          this.renderer.setSize(width, height);
        }
      }
    });
    this.resizeObserver.observe(this.containerRef);

    // 6. Render Loop
    const animate = () => {
      this.animationFrameId = requestAnimationFrame(animate);
      this.controls.update();
      this.renderer.render(this.scene, this.camera);
    };
    animate();
  }

  private loadModel(url: string) {
    const loader = new STLLoader();
    
    loader.load(
      url,
      (geometry) => {
        // Remove previous mesh if updating
        if (this.currentMesh) {
          this.scene.remove(this.currentMesh);
          this.currentMesh.geometry.dispose();
          if (Array.isArray(this.currentMesh.material)) {
            this.currentMesh.material.forEach((mat) => mat.dispose());
          } else {
            this.currentMesh.material.dispose();
          }
        }

        // Center the geometry origin to make rotation natural
        geometry.computeVertexNormals();
        geometry.center();

        const material = new THREE.MeshStandardMaterial({
          color: this.color,
          roughness: 0.4,
          metalness: 0.1,
        });

        this.currentMesh = new THREE.Mesh(geometry, material);
        this.scene.add(this.currentMesh);

        // Auto-adjust camera distance based on object bounding box size
        geometry.computeBoundingSphere();
        const sphere = geometry.boundingSphere;
        if (sphere) {
          const radius = sphere.radius;
          this.camera.position.set(radius * 2.5, radius * 2.5, radius * 2.5);
          this.controls.target.set(0, 0, 0);
          this.controls.update();
        }
      },
      (xhr) => {
        // Optional progress callback: console.log((xhr.loaded / xhr.total) * 100 + '% loaded');
      },
      (error) => {
        console.error('An error happened loading the STL:', error);
      }
    );
  }

  render() {
    return html`
      <div id="canvas-container" style="width: 100%; height: 100%;"></div>
    `;
  }
}