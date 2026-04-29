/* ===================================================================
   CONSTANTS
   =================================================================== */
const SATS_PER_BTC = 100_000_000;

/* ===================================================================
   DOM REFERENCES
   =================================================================== */
// These four elements are owned exclusively by this file.
// External code interacts with them only through resetTransactionFlow()
// and renderTransactionFlow() — never by querying them directly.
const transactionFlow = document.querySelector("#transaction-flow");
const transactionFlowChart = document.querySelector("#transaction-flow-chart");
const transactionFlowInputCount = document.querySelector("#transaction-flow-input-count");
const transactionFlowOutputCount = document.querySelector("#transaction-flow-output-count");

/* ===================================================================
   PUBLIC API
   =================================================================== */
// Called by app.js when returning to the empty state.
function resetTransactionFlow() {
  transactionFlowChart.replaceChildren();
  transactionFlowInputCount.textContent = "0";
  transactionFlowOutputCount.textContent = "0";
  transactionFlow.open = true;
  transactionFlow.hidden = true;
}

// Called by app.js after a successful stamp lookup to render the flow diagram.
function renderTransactionFlow(context, txHash) {
  transactionFlowChart.replaceChildren();
  transactionFlowInputCount.textContent = "0";
  transactionFlowOutputCount.textContent = "0";

  const inputs = normalizeFlowInputs(context?.vin);
  const outputs = normalizeFlowOutputs(context?.vout);

  if (inputs.length === 0 && outputs.length === 0) {
    transactionFlow.hidden = true;
    return;
  }

  transactionFlowInputCount.textContent = inputs.length.toLocaleString("en-US");
  transactionFlowOutputCount.textContent = outputs.length.toLocaleString("en-US");

  const diagram = document.createElement("div");
  const svg = createFlowSvg(inputs, outputs, txHash);

  diagram.className = "transaction-flow-diagram";
  diagram.append(svg);

  transactionFlowChart.append(diagram);
  transactionFlow.open = true;
  transactionFlow.hidden = false;
}

/* ===================================================================
   INPUT / OUTPUT NORMALIZERS
   =================================================================== */
// Reconciles the vin field shapes returned by the Esplora, Blockchain.com,
// and BlockCypher adapters into a uniform { index, label, value, reference } record.
function normalizeFlowInputs(inputs = []) {
  return inputs.map((input, index) => {
    const prevout = input?.prevout || input?.prev_out || {};
    const address =
      prevout.scriptpubkey_address ||
      prevout.address ||
      prevout.addr ||
      input?.scriptpubkey_address ||
      input?.address ||
      input?.addresses?.[0];

    return {
      index,
      label: input?.is_coinbase ? "Coinbase input" : address || "Previous output",
      value: normalizeSats(prevout.value ?? input?.value ?? input?.output_value),
      reference: input?.txid || input?.prev_hash || input?.hash || "",
    };
  });
}

// Reconciles the vout field shapes returned by the Esplora, Blockchain.com,
// and BlockCypher adapters into a uniform { index, label, value, reference } record.
function normalizeFlowOutputs(outputs = []) {
  return outputs.map((output, index) => {
    const script = output?.scriptPubKey || {};
    const address =
      output?.scriptpubkey_address ||
      output?.address ||
      output?.addr ||
      output?.addresses?.[0] ||
      script.address ||
      script.addresses?.[0];

    return {
      index: output?.n ?? index,
      label: address || output?.scriptpubkey_type || script.type || "Output",
      value: normalizeSats(output?.value),
      reference: output?.scriptpubkey_type || script.type || "",
    };
  });
}

/* ===================================================================
   SVG DIAGRAM
   =================================================================== */
// Builds the Sankey-style flow SVG: a single input chevron and trunk on the left,
// fanning out to bezier output lines on the right, with stroke widths proportional
// to each output's sat value relative to the total.
function createFlowSvg(inputs, outputs, txHash) {
  const NS = "http://www.w3.org/2000/svg";
  const W = 1000;
  const TRUNK_H = 80;
  const MIN_LINE_H = 3;
  const MIN_GAP = 2;
  const PREFERRED_GAP = 8;
  const PADDING_Y = 44;
  const INPUT_CX = TRUNK_H / 2; // chevron left edge at x = 0
  const CENTER_X = 200;
  const OUTPUT_X = W;           // output lines reach the right edge

  const totalIn = inputs.reduce((s, i) => s + Math.max(0, i.value ?? 0), 0);
  const totalOut = outputs.reduce((s, o) => s + Math.max(0, o.value ?? 0), 0);
  const refVal = Math.max(totalIn, totalOut, 1);

  const sorted = [...outputs].sort((a, b) => (a.value ?? 0) - (b.value ?? 0));
  const outHeights = sorted.map((o) => {
    const v = Math.max(0, o.value ?? 0);
    return v === 0 ? MIN_LINE_H : Math.max(MIN_LINE_H, (v / refVal) * TRUNK_H);
  });

  const gap = sorted.length > 60 ? MIN_GAP : PREFERRED_GAP;
  const stackH = outHeights.reduce((s, h) => s + h, 0) + gap * Math.max(0, sorted.length - 1);
  const H = Math.max(TRUNK_H * 3, stackH + 2 * PADDING_Y);
  const CENTER_Y = H / 2;
  const MID_X = (CENTER_X + OUTPUT_X) / 2;
  const outYs = flowStack(outHeights, CENTER_Y, gap);

  // Start-Y: map each output center so first edge = trunk top, last edge = trunk bottom.
  // Normalize cumulative position against [h0/2 … totalOutH−hL/2] so frac is exactly
  // 0 for the first output and exactly 1 for the last — guaranteeing flush edges.
  const half = TRUNK_H / 2;
  const totalOutH = outHeights.reduce((s, h) => s + h, 0);
  const h0 = outHeights[0];
  const hL = outHeights[outHeights.length - 1];
  const innerMin = CENTER_Y - half + h0 / 2;
  const innerMax = CENTER_Y + half - hL / 2;
  const innerRange = Math.max(0, innerMax - innerMin);
  const totalInner = Math.max(totalOutH - h0 / 2 - hL / 2, 1);
  let cumH = 0;
  const startYs = outHeights.map((h) => {
    const frac = Math.max(0, Math.min(1, (cumH + h / 2 - h0 / 2) / totalInner));
    cumH += h;
    return innerMin + frac * innerRange;
  });

  const svg = document.createElementNS(NS, "svg");
  const defs = document.createElementNS(NS, "defs");

  const gIn = document.createElementNS(NS, "linearGradient");
  gIn.setAttribute("id", "tf-in");
  gIn.setAttribute("gradientUnits", "userSpaceOnUse");
  gIn.setAttribute("x1", "0"); gIn.setAttribute("x2", String(CENTER_X));
  gIn.setAttribute("y1", "0"); gIn.setAttribute("y2", "0");
  appendGradientStop(gIn, "0%", "var(--color-primary-800)");
  appendGradientStop(gIn, "100%", "var(--color-primary-400)");

  const gOut = document.createElementNS(NS, "linearGradient");
  gOut.setAttribute("id", "tf-out");
  gOut.setAttribute("gradientUnits", "userSpaceOnUse");
  gOut.setAttribute("x1", String(CENTER_X)); gOut.setAttribute("x2", String(W));
  gOut.setAttribute("y1", "0"); gOut.setAttribute("y2", "0");
  appendGradientStop(gOut, "0%", "var(--color-primary-400)");
  appendGradientStop(gOut, "100%", "var(--color-primary-800)");

  defs.append(gIn, gOut);
  svg.append(defs);

  svg.setAttribute("class", "transaction-flow-svg");
  svg.setAttribute("viewBox", `0 0 ${W} ${H}`);
  svg.setAttribute("role", "img");
  svg.setAttribute("aria-labelledby", "transaction-flow-svg-title");

  const titleEl = document.createElementNS(NS, "title");
  titleEl.setAttribute("id", "transaction-flow-svg-title");
  titleEl.textContent = `Transaction ${shortenMiddle(txHash)}: ${formatCount(inputs.length, "input")}, ${formatCount(sorted.length, "output")}.`;
  svg.append(titleEl);

  // Shift trunk left by half/2 so the notch tip sits only 20 px from the chevron tip —
  // while keeping notch depth = half for the same 45° angle as the input chevron.
  const INPUT_RIGHT = INPUT_CX + half / 2;

  // Input chevron — pointing right
  const chevIn = document.createElementNS(NS, "path");
  chevIn.setAttribute("d", `M ${INPUT_CX - half} ${CENTER_Y - half} L ${INPUT_CX} ${CENTER_Y - half} L ${INPUT_CX + half} ${CENTER_Y} L ${INPUT_CX} ${CENTER_Y + half} L ${INPUT_CX - half} ${CENTER_Y + half} Z`);
  chevIn.setAttribute("fill", "url(#tf-in)");
  svg.append(chevIn);

  // Trunk — filled pentagon, left end has > notch with same 45° angle as the input chevron
  const trunk = document.createElementNS(NS, "path");
  trunk.setAttribute("d", `M ${INPUT_RIGHT} ${CENTER_Y - half} L ${INPUT_RIGHT + half} ${CENTER_Y} L ${INPUT_RIGHT} ${CENTER_Y + half} L ${CENTER_X} ${CENTER_Y + half} L ${CENTER_X} ${CENTER_Y - half} Z`);
  trunk.setAttribute("fill", "url(#tf-in)");
  svg.append(trunk);

  // Output bezier curves — start proportionally spread across trunk height, fan to stacked positions
  for (let i = 0; i < sorted.length; i++) {
    const sy = startYs[i];
    const oy = outYs[i];
    const lh = outHeights[i];
    const line = document.createElementNS(NS, "path");
    line.setAttribute("d", `M ${CENTER_X} ${sy} C ${MID_X} ${sy}, ${MID_X} ${oy}, ${OUTPUT_X} ${oy}`);
    line.setAttribute("fill", "none");
    line.setAttribute("stroke", "url(#tf-out)");
    line.setAttribute("stroke-width", String(Math.max(MIN_LINE_H, lh)));
    line.setAttribute("stroke-linecap", "butt");
    svg.append(line);
  }

  return svg;
}

// Converts an array of heights into centred Y positions, evenly spaced by gap.
function flowStack(heights, centerY, gap) {
  const total = heights.reduce((s, h) => s + h, 0) + gap * Math.max(0, heights.length - 1);
  let y = centerY - total / 2;
  return heights.map((h) => {
    const cy = y + h / 2;
    y += h + gap;
    return cy;
  });
}

// Appends a single color stop to an SVG linearGradient element.
function appendGradientStop(gradient, offset, color) {
  const stop = document.createElementNS("http://www.w3.org/2000/svg", "stop");
  stop.setAttribute("offset", offset);
  stop.setAttribute("stop-color", color);
  gradient.append(stop);
}

/* ===================================================================
   FORMATTING UTILITIES
   =================================================================== */
// Normalises a raw value to satoshis. Integer values are assumed to already be
// in satoshis; non-integer values are assumed to be in BTC and are multiplied
// by SATS_PER_BTC to convert.
function normalizeSats(value) {
  if (value === null || value === undefined || value === "") {
    return null;
  }

  const amount = Number(value);

  if (!Number.isFinite(amount)) {
    return null;
  }

  if (Number.isInteger(amount)) {
    return amount;
  }

  return Math.round(amount * SATS_PER_BTC);
}

function formatCount(count, singular) {
  return `${count} ${singular}${count === 1 ? "" : "s"}`;
}

function shortenMiddle(value, visible = 8) {
  const text = String(value || "");

  if (text.length <= visible * 2 + 1) {
    return text;
  }

  return `${text.slice(0, visible)}...${text.slice(-visible)}`;
}
