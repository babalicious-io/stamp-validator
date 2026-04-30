/* ===================================================================
   CONSTANTS & PROVIDERS
   =================================================================== */
const TX_HASH_PATTERN = /^[0-9a-f]{64}$/;

const PROVIDERS = {
  mempool: {
    label: "Mempool",
    baseUrl: "https://mempool.space/api",
    adapter: "esplora",
  },
  blockstream: {
    label: "Blockstream",
    baseUrl: "https://blockstream.info/api",
    adapter: "esplora",
  },
  blockchain: {
    label: "Blockchain",
    baseUrl: "https://blockchain.info",
    adapter: "blockchain",
  },
  blockcypher: {
    label: "BlockCypher",
    baseUrl: "https://api.blockcypher.com/v1/btc/main",
    adapter: "blockcypher",
  },
  blockchair: {
    label: "Blockchair",
    baseUrl: "https://api.blockchair.com/bitcoin",
    adapter: "blockchair",
  },
};

/* ===================================================================
   METADATA KEY LISTS
   =================================================================== */
const HTML_STAMP_DATA_KEYS = ["html_title", "html_author"];
const BLOCK_DETAIL_TOP_KEYS = ["tx_hash"];
const STAMP_MEDIA_DATA_KEYS = ["stamp_mimetype", "file_size_bytes"];

const STAMP_DATA_KEYS = [
  "creator",
  ...STAMP_MEDIA_DATA_KEYS,
  "keyburn",
  "stamp_hash",
  "is_btc_stamp",
];

const SRC20_STAMP_DATA_KEYS = [
  "tick",
  "src20_operation",
  "creator",
  "receiver",
  "stamp_mimetype",
  "file_size_bytes",
  "keyburn",
  "stamp_hash",
  "is_btc_stamp",
];

/* ===================================================================
   DOM REFERENCES
   =================================================================== */
// Note: #transaction-flow* elements are queried and owned by tx-flow-chart.js.
// app.js resets the chart panel via resetTransactionFlow() — see tx-flow-chart.js.
const form = document.querySelector("#stamp-search-form");
const providerSelect = document.querySelector("#node-provider");
const txHashInput = document.querySelector("#tx-hash");
const resultCard = document.querySelector("#stamp-result-card");
const mediaPreview = document.querySelector("#media-preview");
const mediaFrame = document.querySelector("#media-frame");
const stampBase64Panel = document.querySelector("#stamp-base64-panel");
const stampBase64List = document.querySelector("#stamp-base64-list");
const stampDataPanel = document.querySelector("#stamp-data-panel");
const stampDataList = document.querySelector("#stamp-data-list");
const bitcoinDataList = document.querySelector("#bitcoin-data-list");
const emptyState = document.querySelector("#empty-state");
const submitButton = form.querySelector("button[type='submit']");

/* ===================================================================
   MUTABLE STATE
   =================================================================== */
let wasmIndexer = null;
let activeRequestId = 0;
let activeController = null;
let statusMessage = null;
let statusMessageTimeout = null;

/* ===================================================================
   VALIDATION
   =================================================================== */
function normalizeTxHash(value) {
  return value.trim().toLowerCase();
}

function validateTxHash(value) {
  const normalized = normalizeTxHash(value);

  if (!normalized) {
    return {
      isValid: false,
      message: "Enter a Bitcoin transaction hash.",
      normalized,
    };
  }

  if (normalized.length !== 64) {
    return {
      isValid: false,
      message: "Transaction hashes must be exactly 64 characters.",
      normalized,
    };
  }

  if (!TX_HASH_PATTERN.test(normalized)) {
    return {
      isValid: false,
      message: "Transaction hashes can only contain hexadecimal characters.",
      normalized,
    };
  }

  return {
    isValid: true,
    message: "",
    normalized,
  };
}

/* ===================================================================
   STATUS TOAST
   =================================================================== */
function getStatusMessage() {
  if (!statusMessage) {
    statusMessage = document.createElement("div");
    statusMessage.className = "message";
    statusMessage.setAttribute("aria-atomic", "true");
    document.body.append(statusMessage);
  }

  return statusMessage;
}

function removeStatusMessage(statusElement) {
  statusElement.remove();
  if (statusMessage === statusElement) {
    statusMessage = null;
  }
}

function setStatus(message, type = "", autoDismiss = type !== "loading") {
  if (statusMessageTimeout) {
    clearTimeout(statusMessageTimeout);
    statusMessageTimeout = null;
  }

  if (!message && !statusMessage) {
    return;
  }

  const statusElement = getStatusMessage();
  statusElement.getAnimations().forEach((animation) => animation.cancel());

  statusElement.textContent = message;
  statusElement.setAttribute("role", type === "error" ? "alert" : "status");
  statusElement.setAttribute("aria-live", type === "error" ? "assertive" : "polite");
  statusElement.classList.remove("success", "error", "loading");

  if (type) {
    statusElement.classList.add(type);
  }

  if (!message) {
    statusElement.style.opacity = "0";
    statusElement.style.transform = "translateY(-20px)";
    removeStatusMessage(statusElement);
    return;
  }

  statusElement.style.opacity = "1";
  statusElement.style.transform = "translateY(0)";
  statusElement.animate(
    [
      { opacity: 0, transform: "translateY(-20px)" },
      { opacity: 1, transform: "translateY(0)" },
    ],
    {
      duration: 300,
      easing: "ease-out",
      fill: "forwards",
    },
  );

  if (!autoDismiss) {
    return;
  }

  statusMessageTimeout = setTimeout(() => {
    const fadeOut = statusElement.animate(
      [
        { opacity: 1, transform: "translateY(0)" },
        { opacity: 0, transform: "translateY(-20px)" },
      ],
      {
        duration: 300,
        easing: "ease-in",
        fill: "forwards",
      },
    );

    fadeOut.finished
      .then(() => {
        removeStatusMessage(statusElement);
        statusMessageTimeout = null;
      })
      .catch(() => {});
  }, 3000);
}

/* ===================================================================
   UI STATE
   =================================================================== */
function setFieldValidity(message) {
  txHashInput.setAttribute("aria-invalid", message ? "true" : "false");
}

function setLoading(isLoading) {
  submitButton.disabled = isLoading;
  providerSelect.disabled = isLoading;
  txHashInput.disabled = isLoading;
}

function showEmptyState() {
  resultCard.hidden = true;
  emptyState.hidden = false;
  mediaFrame.replaceChildren();
  mediaPreview.hidden = true;
  resetTransactionFlow();
  stampBase64Panel.open = false;
  stampBase64Panel.hidden = true;
  stampBase64List.replaceChildren();
  stampDataPanel.open = true;
  stampDataList.replaceChildren();
  bitcoinDataList.replaceChildren();
}

function showResult(result, context, txHash) {
  const metadata = result.metadata;
  const shouldDisplayStampBase64 = !hasSrc20Identifier(metadata);
  const shouldDisplayHtmlStampDetails = hasHtmlStampMedia(result.media, metadata);

  renderMedia(result.media);
  renderStampBase64(metadata, shouldDisplayStampBase64);
  renderTransactionFlow(context, txHash);
  renderStampDataMetadata(metadata, shouldDisplayHtmlStampDetails, result.srcProtocol);
  renderLayer1Metadata(metadata, shouldDisplayHtmlStampDetails);
  resultCard.hidden = false;
  emptyState.hidden = true;
}

/* ===================================================================
   WASM/RUST INDEXER
   =================================================================== */
async function loadWasmIndexer() {
  try {
    const response = await fetch("./app.wasm");
    const bytes = await response.arrayBuffer();
    const wasmModule = await WebAssembly.instantiate(bytes, {});
    const exports = wasmModule.instance.exports;

    if (!exports.memory || !exports.alloc || !exports.dealloc || !exports.index_stamp) {
      throw new Error("Wasm module does not expose the stamp indexer ABI.");
    }

    return {
      exports,
      indexStamp(input) {
        const inputJson = JSON.stringify(input);
        const encoded = new TextEncoder().encode(inputJson);
        const inputPtr = exports.alloc(encoded.length);
        new Uint8Array(exports.memory.buffer, inputPtr, encoded.length).set(encoded);

        const packed = exports.index_stamp(inputPtr, encoded.length);
        exports.dealloc(inputPtr, encoded.length);

        // index_stamp returns a packed i64: high 32 bits = output pointer,
        // low 32 bits = output byte length. BigInt arithmetic handles the
        // full 64-bit range safely across all JS engines.
        const packedBigInt = typeof packed === "bigint" ? packed : BigInt(packed);
        const outputPtr = Number(packedBigInt >> 32n);
        const outputLen = Number(packedBigInt & 0xffffffffn);
        const outputBytes = new Uint8Array(exports.memory.buffer, outputPtr, outputLen);
        const outputJson = new TextDecoder().decode(outputBytes);
        exports.dealloc(outputPtr, outputLen);
        return JSON.parse(outputJson);
      },
    };
  } catch {
    return null;
  }
}

/* ===================================================================
   NETWORK — FETCH DISPATCH
   =================================================================== */
async function fetchTransactionData(providerKey, txHash, signal) {
  const provider = PROVIDERS[providerKey] || PROVIDERS.mempool;

  if (provider.adapter === "blockchain") {
    return fetchBlockchainTransactionData(provider, txHash, signal);
  }

  if (provider.adapter === "blockcypher") {
    return fetchBlockCypherTransactionData(provider, txHash, signal);
  }

  if (provider.adapter === "blockchair") {
    return fetchBlockchairTransactionData(provider, txHash, signal);
  }

  return fetchEsploraTransactionData(provider, txHash, signal);
}

/* ===================================================================
   NETWORK — PROVIDER ADAPTERS
   =================================================================== */
async function fetchEsploraTransactionData(provider, txHash, signal) {
  const txUrl = `${provider.baseUrl}/tx/${txHash}`;
  const hexUrl = `${txUrl}/hex`;
  const tipHeightUrl = `${provider.baseUrl}/blocks/tip/height`;
  const [contextResponse, rawTxResponse] = await Promise.all([
    fetch(txUrl, { signal }),
    fetch(hexUrl, { signal }),
  ]);

  ensureOk(contextResponse, provider);
  ensureOk(rawTxResponse, provider);

  return {
    provider,
    context: normalizeEsploraContext(
      await contextResponse.json(),
      await fetchOptionalText(tipHeightUrl, signal),
    ),
    rawTxHex: (await rawTxResponse.text()).trim(),
  };
}

async function fetchBlockchainTransactionData(provider, txHash, signal) {
  const txUrl = `${provider.baseUrl}/rawtx/${txHash}?cors=true`;
  const hexUrl = `${provider.baseUrl}/rawtx/${txHash}?format=hex&cors=true`;
  const latestBlockUrl = `${provider.baseUrl}/latestblock?cors=true`;
  const [contextResponse, rawTxResponse] = await Promise.all([
    fetch(txUrl, { signal }),
    fetch(hexUrl, { signal }),
  ]);

  ensureOk(contextResponse, provider);
  ensureOk(rawTxResponse, provider);

  const context = await contextResponse.json();
  const rawTxHex = (await rawTxResponse.text()).trim();
  const latestBlock = await fetchOptionalJson(latestBlockUrl, signal);

  return {
    provider,
    context: normalizeBlockchainContext(context, latestBlock),
    rawTxHex,
  };
}

async function fetchBlockCypherTransactionData(provider, txHash, signal) {
  const response = await fetch(`${provider.baseUrl}/txs/${txHash}?includeHex=true`, {
    signal,
  });

  ensureOk(response, provider);

  const context = await response.json();
  const rawTxHex = typeof context.hex === "string" ? context.hex.trim() : "";

  if (!rawTxHex) {
    throw new Error(`${provider.label} returned transaction data without raw hex.`);
  }

  return {
    provider,
    context: normalizeBlockCypherContext(context),
    rawTxHex,
  };
}

async function fetchBlockchairTransactionData(provider, txHash, signal) {
  const response = await fetch(`${provider.baseUrl}/raw/transaction/${txHash}`, {
    signal,
  });

  ensureOk(response, provider);

  const context = await response.json();
  const transaction = context.data?.[txHash];
  const rawTxHex =
    typeof transaction?.raw_transaction === "string"
      ? transaction.raw_transaction.trim()
      : "";

  if (!rawTxHex) {
    throw new Error(`${provider.label} returned transaction data without raw hex.`);
  }

  return {
    provider,
    context: normalizeBlockchairContext(context, txHash),
    rawTxHex,
  };
}

function ensureOk(response, provider) {
  if (!response.ok) {
    throw new Error(`${provider.label} could not return transaction data for this hash.`);
  }
}

async function fetchOptionalText(url, signal) {
  try {
    const response = await fetch(url, { signal });

    return response.ok ? await response.text() : null;
  } catch (error) {
    if (error.name === "AbortError") {
      throw error;
    }

    return null;
  }
}

async function fetchOptionalJson(url, signal) {
  try {
    const response = await fetch(url, { signal });

    return response.ok ? await response.json() : null;
  } catch (error) {
    if (error.name === "AbortError") {
      throw error;
    }

    return null;
  }
}

/* ===================================================================
   NETWORK — CONTEXT NORMALIZERS
   =================================================================== */
// Esplora (mempool.space, Blockstream): native esplora tx object.
function normalizeEsploraContext(context, tipHeight) {
  return {
    ...context,
    localTxStats: {
      fee_sats: numberOrNull(context.fee),
      vsize: numberOrNull(context.vsize),
      chain_tip_height: numberOrNull(tipHeight),
    },
  };
}

// Blockchain.com: maps its `inputs`/`out` shape to the esplora-compatible vin/vout format.
function normalizeBlockchainContext(context, latestBlock) {
  return {
    ...context,
    status: {
      block_height: context.block_height ?? null,
      block_time: context.time ?? null,
      confirmed: Boolean(context.block_height),
    },
    vin: (context.inputs || []).map((input) => ({
      txid: input.prev_out?.tx_hash,
      vout: input.prev_out?.n,
      prevout: {
        scriptpubkey_address:
          input.prev_out?.addr ||
          input.prev_out?.address ||
          input.prev_out?.scriptpubkey_address,
        value: input.prev_out?.value,
      },
    })),
    vout: (context.out || []).map((output) => ({
      n: output.n,
      scriptpubkey_address:
        output.addr || output.address || output.scriptpubkey_address,
      value: output.value,
    })),
    localTxStats: {
      fee_sats: numberOrNull(context.fee),
      vsize: numberOrNull(context.vsize),
      chain_tip_height: numberOrNull(latestBlock?.height),
    },
  };
}

// BlockCypher: maps its `inputs`/`outputs` shape and ISO-8601 confirmed timestamp.
function normalizeBlockCypherContext(context) {
  const blockTime = context.confirmed
    ? Math.floor(Date.parse(context.confirmed) / 1000)
    : null;

  return {
    ...context,
    status: {
      block_height: context.block_height ?? null,
      block_time: Number.isNaN(blockTime) ? null : blockTime,
      confirmed: context.confirmations > 0,
    },
    vin: (context.inputs || []).map((input) => ({
      txid: input.prev_hash,
      vout: input.output_index,
      prevout: {
        scriptpubkey_address: input.addresses?.[0],
        value: input.output_value,
      },
    })),
    vout: (context.outputs || []).map((output, index) => ({
      n: output.n ?? index,
      scriptpubkey_address: output.addresses?.[0],
      value: output.value,
    })),
    localTxStats: {
      fee_sats: numberOrNull(context.fees),
      vsize: numberOrNull(context.vsize),
    },
  };
}

// Blockchair: extracts the transaction record from the nested data envelope.
function normalizeBlockchairContext(context, txHash) {
  const txData = context.data?.[txHash] || {};
  const transaction = txData.transaction || txData;

  return {
    ...context,
    status: {
      block_height: transaction.block_id ?? null,
      block_time: parseBlockchairTime(transaction.time),
      confirmed: Boolean(transaction.block_id),
    },
    localTxStats: {
      fee_sats: numberOrNull(transaction.fee),
      vsize: numberOrNull(transaction.virtual_size ?? transaction.vsize),
      confirmations: numberOrNull(transaction.confirmations),
    },
  };
}

function parseBlockchairTime(value) {
  if (typeof value !== "string") {
    return null;
  }

  const timestamp = Date.parse(`${value.replace(" ", "T")}Z`);
  return Number.isNaN(timestamp) ? null : Math.floor(timestamp / 1000);
}

function numberOrNull(value) {
  const number = Number(value);
  return Number.isFinite(number) ? number : null;
}

/* ===================================================================
   METADATA — DATA HELPERS
   =================================================================== */
function hasSrc20Identifier(fields = []) {
  const identifier = fields.find((field) => field.key === "ident" && hasMetadataValue(field));

  if (!identifier) {
    return false;
  }

  return normalizeIdentifier(identifier.value) === "SRC20";
}

function getSrc20Operation(fields = []) {
  if (!hasSrc20Identifier(fields)) {
    return "";
  }

  const operation = fields.find(
    (field) => field.key === "src20_operation" && hasMetadataValue(field),
  )?.value;

  return typeof operation === "string" ? operation.toLowerCase() : "";
}

function hasHtmlStampMedia(media, fields = []) {
  if (media?.kind === "html") {
    return true;
  }

  const mimetype = fields.find((field) => field.key === "stamp_mimetype" && hasMetadataValue(field));

  return typeof mimetype?.value === "string" && mimetype.value.toLowerCase().includes("html");
}

function getStampDataKeys({ hasHtmlStampDetails = false, hasSrc20 = false } = {}) {
  if (hasSrc20) {
    return SRC20_STAMP_DATA_KEYS;
  }

  if (hasHtmlStampDetails) {
    return [
      ...HTML_STAMP_DATA_KEYS,
      "creator",
      ...STAMP_MEDIA_DATA_KEYS,
      "keyburn",
      "stamp_hash",
      "is_btc_stamp",
    ];
  }

  return STAMP_DATA_KEYS;
}

function insertSrc20ProtocolFields(fields, protocolFields) {
  if (protocolFields.length === 0) {
    return fields;
  }

  const transactionTypeIndex = fields.findIndex((field) => field.key === "src20_operation");

  if (transactionTypeIndex === -1) {
    return [...protocolFields, ...fields];
  }

  return [
    ...fields.slice(0, transactionTypeIndex + 1),
    ...protocolFields,
    ...fields.slice(transactionTypeIndex + 1),
  ];
}

function getSrc20ProtocolFields(srcProtocol, operation) {
  if (!srcProtocol || typeof srcProtocol !== "object") {
    return [];
  }

  if (operation === "transfer" || operation === "mint") {
    const amount = getProtocolValue(srcProtocol, "amt");

    return hasProtocolValue(amount)
      ? [{ key: "src20_amount", label: "Amount", value: amount }]
      : [];
  }

  if (operation === "deploy") {
    return [
      { key: "src20_max_supply", label: "Max Supply", value: getProtocolValue(srcProtocol, "max") },
      { key: "src20_limit_amount", label: "Limit Amount", value: getProtocolValue(srcProtocol, "lim") },
    ].filter(hasMetadataValue);
  }

  return [];
}

function getProtocolValue(srcProtocol, key) {
  return srcProtocol[key] ?? srcProtocol[key.toUpperCase()] ?? null;
}

function hasProtocolValue(value) {
  return value !== null && value !== undefined && value !== "";
}

function getSrc20CreatorLabel(operation) {
  if (operation === "transfer") {
    return "Sender Addy";
  }

  if (operation === "deploy") {
    return "Creator Addy";
  }

  if (operation === "mint") {
    return "Mint Addy";
  }

  return "Artist Addy";
}

function hasMetadataValue(field) {
  const { value } = field;

  if (value === null || value === undefined || value === "") {
    return false;
  }

  if (Array.isArray(value)) {
    return value.length > 0;
  }

  return true;
}

/* ===================================================================
   METADATA — RENDERING
   =================================================================== */
function renderLayer1Metadata(fields = [], hasHtmlStampDetails = false) {
  const hasSrc20 = hasSrc20Identifier(fields);
  const stampDataKeys = getStampDataKeys({ hasHtmlStampDetails, hasSrc20 });
  const confirmationsField = fields.find((field) => field.key === "confirmations" && hasMetadataValue(field));
  const layer1TopFields = BLOCK_DETAIL_TOP_KEYS.map((key) =>
    fields.find((field) => field.key === key && hasMetadataValue(field)),
  ).filter(Boolean);
  const layer1Fields = fields.filter(
    (field) =>
      ![
        "encoding_method",
        "stamp_base64",
        "file_hash",
        "is_valid_base64",
        "input_count",
        "output_count",
        "ident",
        "confirmations",
        ...BLOCK_DETAIL_TOP_KEYS,
        ...stampDataKeys,
      ].includes(field.key) &&
      hasMetadataValue(field),
  );
  const layer1MetadataFields = [...layer1TopFields, ...layer1Fields];

  renderMetadata(bitcoinDataList, layer1MetadataFields, {
    // Attach a confirmations pill badge to the tx_hash row when both fields are present.
    pill:
      confirmationsField && layer1MetadataFields.some((field) => field.key === "tx_hash")
        ? {
            targetKey: "tx_hash",
            value: formatMetadataValue(confirmationsField.value, confirmationsField.key),
          }
        : null,
  });
}

function renderStampDataMetadata(fields = [], hasHtmlStampDetails = false, srcProtocol = null) {
  const hasSrc20 = hasSrc20Identifier(fields);
  const src20Operation = getSrc20Operation(fields);
  const stampDataKeys = getStampDataKeys({ hasHtmlStampDetails, hasSrc20 });
  const identifierField = fields.find((field) => field.key === "ident" && hasMetadataValue(field));
  const identifierPillTarget = hasSrc20 ? "tick" : hasHtmlStampDetails ? "html_title" : "creator";
  const baseStampDataFields = stampDataKeys.map((key) =>
    fields.find((field) => field.key === key && hasMetadataValue(field)),
  ).filter(Boolean).map((field) =>
    field.key === "creator" && hasSrc20
      ? { ...field, label: getSrc20CreatorLabel(src20Operation) }
      : field,
  );
  const stampDataFields = insertSrc20ProtocolFields(
    baseStampDataFields,
    getSrc20ProtocolFields(srcProtocol, src20Operation),
  );
  stampDataPanel.open = true;
  renderMetadata(stampDataList, stampDataFields, {
    // Attach the ident badge (e.g. "SRC-20") to the primary identifier row.
    pill:
      identifierField && stampDataFields.some((field) => field.key === identifierPillTarget)
        ? {
            targetKey: identifierPillTarget,
            value: formatMetadataValue(identifierField.value, identifierField.key),
          }
        : null,
  });
}

function renderStampBase64(fields = [], shouldDisplay = true) {
  const stampBase64Field = fields.find(
    (field) => field.key === "stamp_base64" && hasMetadataValue(field),
  );
  const stampBase64Fields = ["encoding_method", "stamp_base64", "file_hash", "is_valid_base64"]
    .map((key) => fields.find((field) => field.key === key && hasMetadataValue(field)))
    .filter(Boolean);

  stampBase64List.replaceChildren();
  stampBase64Panel.open = false;

  if (!shouldDisplay || !stampBase64Field) {
    stampBase64Panel.hidden = true;
    return;
  }

  renderMetadata(stampBase64List, stampBase64Fields);
  stampBase64Panel.hidden = false;
}

function renderMetadata(list, fields = [], options = {}) {
  list.replaceChildren();

  for (const field of fields) {
    const row = document.createElement("div");
    const term = document.createElement("dt");
    const description = document.createElement("dd");

    term.textContent = formatMetadataLabel(field);
    description.textContent = formatMetadataValue(field.value, field.key);

    // When a pill badge is configured for this field's key, wrap the label
    // in a flex row and append the badge span alongside it.
    if (options.pill?.targetKey === field.key) {
      const pill = document.createElement("span");

      term.className = "container-row container-row--center result-list-row-header";
      pill.className = "container-pill";
      pill.textContent = options.pill.value;
      term.append(pill);
      row.append(term, description);
    } else {
      row.append(term, description);
    }

    list.append(row);
  }
}

/* ===================================================================
   METADATA — FORMATTING
   =================================================================== */
function formatMetadataLabel(field) {
  if (field.key === "html_author") {
    return "Artist";
  }

  if (field.key === "tick") {
    return "Token Ticker";
  }

  if (field.key === "src20_operation") {
    return "Transaction Type";
  }

  if (field.key === "creator") {
    return field.label && field.label !== "Creator" ? field.label : "Artist Addy";
  }

  if (field.key === "receiver") {
    return "Receiver Addy";
  }

  if (field.key === "file_size_bytes") {
    return "File Size";
  }

  return field.label;
}

function formatMetadataValue(value, key) {
  if (value === null || value === undefined || value === "") {
    return "Not available from local transaction data";
  }

  if (key === "ident" && normalizeIdentifier(value) === "STAMP") {
    return "CLASSIC";
  }

  if (typeof value === "boolean") {
    return value ? "True" : "False";
  }

  if (value === "true" || value === "false") {
    return value === "true" ? "True" : "False";
  }

  if (key === "keyburn" && (value === 0 || value === 1 || value === "0" || value === "1")) {
    return Number(value) === 1 ? "True" : "False";
  }

  if (key === "block_time") {
    return formatBlockDate(value);
  }

  if (key === "fee_sats") {
    return `${Number(value).toLocaleString("en-US")} Sats`;
  }

  if (key === "vsize") {
    return `${Number(value).toLocaleString("en-US")} vB`;
  }

  if (key === "confirmations") {
    const confirmations = Number(value);

    return Number.isFinite(confirmations)
      ? `${confirmations.toLocaleString("en-US")} ${confirmations === 1 ? "Confirmation" : "Confirmations"}`
      : String(value);
  }

  if (key === "file_size_bytes") {
    return `${Number(value).toLocaleString("en-US")} Bytes`;
  }

  if (key === "stamp_mimetype") {
    return capitalizeFirstLetter(value);
  }

  if (key === "src20_operation") {
    return capitalizeFirstLetter(value);
  }

  if (typeof value === "object") {
    return JSON.stringify(value);
  }

  return String(value);
}

function capitalizeFirstLetter(value) {
  const text = String(value);
  return text ? `${text.charAt(0).toUpperCase()}${text.slice(1)}` : text;
}

function normalizeIdentifier(value) {
  return String(value).replace(/[^a-z0-9]/gi, "").toUpperCase();
}

function formatBlockDate(value) {
  const timestamp = Number(value);

  if (!Number.isFinite(timestamp)) {
    return String(value);
  }

  const date = new Date(timestamp * 1000);

  if (Number.isNaN(date.getTime())) {
    return String(value);
  }

  const day = String(date.getUTCDate()).padStart(2, "0");
  const month = String(date.getUTCMonth() + 1).padStart(2, "0");
  const year = date.getUTCFullYear();

  return `${day}-${month}-${year}`;
}

/* ===================================================================
   MEDIA PREVIEW
   =================================================================== */
function renderMedia(media) {
  mediaFrame.replaceChildren();

  if (!media || media.kind === "none") {
    mediaPreview.hidden = true;
    return;
  }

  if (media.kind === "image" && media.dataUrl) {
    const image = document.createElement("img");
    image.src = media.dataUrl;
    image.alt = "Decoded Bitcoin Stamp media";
    mediaFrame.append(image);
  } else if (media.kind === "html" && media.dataUrl) {
    const iframe = document.createElement("iframe");
    iframe.title = "Decoded Bitcoin Stamp HTML";
    iframe.sandbox = "allow-scripts";
    iframe.referrerPolicy = "no-referrer";
    iframe.src = media.dataUrl;
    mediaFrame.append(iframe);
  } else {
    const pre = document.createElement("pre");
    pre.className = "media-text";
    pre.textContent = formatMediaText(media);
    mediaFrame.append(pre);
  }

  mediaPreview.hidden = false;
}

function formatMediaText(media) {
  const text = media.text || media.base64 || "Binary media detected.";

  if (media.kind !== "json" || typeof text !== "string") {
    return text;
  }

  try {
    const json = JSON.parse(text);
    return JSON.stringify(json, null, 2);
  } catch {
    return text;
  }
}

/* ===================================================================
   EVENT HANDLERS
   =================================================================== */
async function handleSearch() {
  const validation = validateTxHash(txHashInput.value);
  setFieldValidity(validation.message);

  if (!validation.isValid) {
    setStatus(validation.message, "error");
    showEmptyState();
    return;
  }

  if (!wasmIndexer) {
    showEmptyState();
    setStatus(
      "Rust/Wasm stamp indexer is not available. Build app.wasm from indexer first.",
      "error",
    );
    return;
  }

  // Increment request ID and abort any in-flight request before starting a new one.
  // After each await, stale requests are discarded by comparing against activeRequestId.
  activeRequestId += 1;
  const requestId = activeRequestId;
  activeController?.abort();
  activeController = new AbortController();
  setLoading(true);
  setStatus("Fetching raw transaction data...", "loading");

  try {
    const providerKey = providerSelect.value;
    const { provider, rawTxHex, context } = await fetchTransactionData(
      providerKey,
      validation.normalized,
      activeController.signal,
    );

    if (requestId !== activeRequestId) {
      return;
    }

    setStatus("Processing transaction locally in Rust/Wasm...", "loading");
    const result = wasmIndexer.indexStamp({
      txHash: validation.normalized,
      provider: provider.label,
      rawTxHex,
      context,
    });

    if (requestId !== activeRequestId) {
      return;
    }

    showResult(result, context, validation.normalized);
    setStatus(result.message, "success");
  } catch (error) {
    if (error.name === "AbortError") {
      return;
    }

    showEmptyState();
    setStatus(error.message || "Stamp lookup failed.", "error");
  } finally {
    if (requestId === activeRequestId) {
      setLoading(false);
    }
  }
}

form.addEventListener("submit", (event) => {
  event.preventDefault();
  handleSearch();
});

txHashInput.addEventListener("input", () => {
  if (txHashInput.getAttribute("aria-invalid") === "true") {
    const validation = validateTxHash(txHashInput.value);
    setFieldValidity(validation.isValid ? "" : validation.message);
    setStatus(
      validation.isValid ? "Transaction hash format looks valid." : validation.message,
      validation.isValid ? "success" : "error",
    );
  }
});

/* ===================================================================
   INITIALISATION
   =================================================================== */
loadWasmIndexer().then((indexer) => {
  wasmIndexer = indexer;
  setStatus(
    wasmIndexer
      ? "Stamp indexer loaded."
      : "Build the stamp indexer to enable lookup.",
    wasmIndexer ? "success" : "error",
  );
});
