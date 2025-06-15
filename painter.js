const canvas = document.getElementById('canvas');
const ctx = canvas.getContext("2d", {desynchronized: true, alpha: false});
const tau = 2 * Math.PI;
export function line_segment(a, b, x, y, w, c) {
    ctx.beginPath();
    ctx.strokeStyle = c;
    ctx.lineWidth = w;
    ctx.moveTo(a, b);
    ctx.lineTo(x, y);
    ctx.stroke();
}
export function circle(x, y, r1, r2, w, c) {
    ctx.beginPath();
    ctx.strokeStyle = c;
    ctx.lineWidth = w;
    ctx.ellipse(x, y, r1, r2, 0, 0, tau);
    ctx.stroke();
}
export function fill(c) {
    ctx.fillStyle = c;
    ctx.fillRect(0, 0, canvas.width, canvas.height);
}
export function fill_rect(a, b, x, y, c) {
    ctx.fillStyle = c;
    ctx.fillRect(a, b, x, y);
}
export function text_bounds(s) {
    ctx.font = "18px monospace";
    const m = ctx.measureText(s);
    const height = m.fontBoundingBoxAscent + m.fontBoundingBoxDescent;
    return [m.width, height];
}
export function fill_text(s, x, y, c) {
    ctx.font = "18px monospace";
    ctx.fillStyle = c;
    ctx.fillText(s, x, y - 4);
}
export function image(rawPixels, width, height, dx, dy, dw, dh, smoothing) {
  const imageData = new ImageData(new Uint8ClampedArray(rawPixels), width, height);
  const offscreen = document.createElement("canvas");
  offscreen.width = width;
  offscreen.height = height;
  const offCtx = offscreen.getContext("2d");
  offCtx.putImageData(imageData, 0, 0);
  ctx.imageSmoothingEnabled = smoothing;
  ctx.drawImage(offscreen, 0, 0, width, height, dx, dy, dw, dh);
}
export function draw(slice, width) {
    const clamped = new Uint8ClampedArray(slice);
    const height = clamped.length / (width * 4);
    const image = new ImageData(clamped, width, height);
    ctx.putImageData(image, 0, 0);
}
export function get_canvas() {
    return canvas;
}
export function resize(x, y) {
    canvas.width = x;
    canvas.height = y;
}
export function dpr() {
    return window.devicePixelRatio
}
export function write_clipboard(text) {
    navigator.clipboard.writeText(text).catch(() => {});
}
