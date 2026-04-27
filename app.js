const TX_HASH_PATTERN = /^[0-9a-f]{64}$/;
const PROVIDERS = {
  mempool: {
    label: "mempool.space",
    baseUrl: "https://mempool.space/api",
    adapter: "esplora",
  },
  blockstream: {
    label: "Blockstream",
    baseUrl: "https://blockstream.info/api",
    adapter: "esplora",
  },
  blockchain: {
    label: "Blockchain.com",
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

const form = document.querySelector("#stamp-search-form");
const providerSelect = document.querySelector("#node-provider");
const txHashInput = document.querySelector("#tx-hash");
const txHashError = document.querySelector("#tx-hash-error");
const statusMessage = document.querySelector("#status-message");
const resultCard = document.querySelector("#stamp-result-card");
const resultMessage = document.querySelector("#result-message");
const resultProvider = document.querySelector("#result-provider");
const mediaPreview = document.querySelector("#media-preview");
const mediaFrame = document.querySelector("#media-frame");
const mediaCaption = document.querySelector("#media-caption");
const metadataList = document.querySelector("#metadata-list");
const emptyState = document.querySelector("#empty-state");
const submitButton = form.querySelector("button[type='submit']");

let wasmIndexer = null;
let activeRequestId = 0;
let activeController = null;

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

function setStatus(message) {
  statusMessage.textContent = message;
}

function setFieldError(message) {
  txHashError.textContent = message;
  txHashInput.setAttribute("aria-invalid", message ? "true" : "false");
}

function showEmptyState() {
  resultCard.hidden = true;
  emptyState.hidden = false;
  resultMessage.textContent = "";
  resultProvider.textContent = "";
  mediaFrame.replaceChildren();
  mediaPreview.hidden = true;
  metadataList.replaceChildren();
}

function showResult(result, providerLabel) {
  resultMessage.textContent = result.message;
  resultProvider.textContent = `Transaction source: ${providerLabel}. Stamp data processed locally by Rust/Wasm.`;
  renderMedia(result.media);
  renderMetadata(result.metadata);
  resultCard.hidden = false;
  emptyState.hidden = true;
}

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

async function fetchEsploraTransactionData(provider, txHash, signal) {
  const txUrl = `${provider.baseUrl}/tx/${txHash}`;
  const hexUrl = `${txUrl}/hex`;
  const [contextResponse, rawTxResponse] = await Promise.all([
    fetch(txUrl, { signal }),
    fetch(hexUrl, { signal }),
  ]);

  ensureOk(contextResponse, provider);
  ensureOk(rawTxResponse, provider);

  return {
    provider,
    context: await contextResponse.json(),
    rawTxHex: (await rawTxResponse.text()).trim(),
  };
}

async function fetchBlockchainTransactionData(provider, txHash, signal) {
  const txUrl = `${provider.baseUrl}/rawtx/${txHash}?cors=true`;
  const hexUrl = `${provider.baseUrl}/rawtx/${txHash}?format=hex&cors=true`;
  const [contextResponse, rawTxResponse] = await Promise.all([
    fetch(txUrl, { signal }),
    fetch(hexUrl, { signal }),
  ]);

  ensureOk(contextResponse, provider);
  ensureOk(rawTxResponse, provider);

  const context = await contextResponse.json();
  const rawTxHex = (await rawTxResponse.text()).trim();

  return {
    provider,
    context: normalizeBlockchainContext(context),
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
    context,
    rawTxHex,
  };
}

function ensureOk(response, provider) {
  if (!response.ok) {
    throw new Error(`${provider.label} could not return transaction data for this hash.`);
  }
}

function normalizeBlockchainContext(context) {
  const firstInput = context.inputs?.[0];
  const creatorAddress =
    firstInput?.prev_out?.addr ||
    firstInput?.prev_out?.address ||
    firstInput?.prev_out?.scriptpubkey_address;

  return {
    ...context,
    status: {
      block_height: context.block_height ?? null,
      block_time: context.time ?? null,
      confirmed: Boolean(context.block_height),
    },
    vin: creatorAddress
      ? [
          {
            prevout: {
              scriptpubkey_address: creatorAddress,
            },
          },
        ]
      : [],
  };
}

function normalizeBlockCypherContext(context) {
  const creatorAddress = context.inputs?.[0]?.addresses?.[0];
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
    vin: creatorAddress
      ? [
          {
            prevout: {
              scriptpubkey_address: creatorAddress,
            },
          },
        ]
      : [],
  };
}

function renderMetadata(fields = []) {
  metadataList.replaceChildren();

  for (const field of fields) {
    const row = document.createElement("div");
    const term = document.createElement("dt");
    const description = document.createElement("dd");
    const source = document.createElement("span");

    term.textContent = field.label;
    description.textContent = formatMetadataValue(field.value);
    source.className = "metadata-source";
    source.textContent = `Source: ${field.source}`;

    description.append(source);
    row.append(term, description);
    metadataList.append(row);
  }
}

function formatMetadataValue(value) {
  if (value === null || value === undefined || value === "") {
    return "Not available from local transaction data";
  }

  if (typeof value === "object") {
    return JSON.stringify(value);
  }

  return String(value);
}

function renderMedia(media) {
  mediaFrame.replaceChildren();
  mediaCaption.textContent = "";

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
    pre.textContent = media.text || media.base64 || "Binary media detected.";
    mediaFrame.append(pre);
  }

  mediaCaption.textContent = media.mimetype
    ? `Decoded media type: ${media.mimetype}`
    : "Decoded media from local stamp payload.";
  mediaPreview.hidden = false;
}

function setLoading(isLoading) {
  submitButton.disabled = isLoading;
  providerSelect.disabled = isLoading;
  txHashInput.disabled = isLoading;
}

form.addEventListener("submit", async (event) => {
  event.preventDefault();

  const validation = validateTxHash(txHashInput.value);
  setFieldError(validation.message);

  if (!validation.isValid) {
    showEmptyState();
    setStatus("Fix the transaction hash and try again.");
    return;
  }

  if (!wasmIndexer) {
    showEmptyState();
    setStatus("Rust/Wasm stamp indexer is not available. Build app.wasm from rust-indexer first.");
    return;
  }

  activeRequestId += 1;
  const requestId = activeRequestId;
  activeController?.abort();
  activeController = new AbortController();
  setLoading(true);
  setStatus("Fetching raw transaction data...");

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

    setStatus("Processing transaction locally in Rust/Wasm...");
    const result = wasmIndexer.indexStamp({
      txHash: validation.normalized,
      provider: provider.label,
      rawTxHex,
      context,
    });

    if (requestId !== activeRequestId) {
      return;
    }

    showResult(result, provider.label);
    setStatus(result.message);
  } catch (error) {
    if (error.name === "AbortError") {
      return;
    }

    showEmptyState();
    setStatus(error.message || "Stamp lookup failed.");
  } finally {
    if (requestId === activeRequestId) {
      setLoading(false);
    }
  }
});

txHashInput.addEventListener("input", () => {
  if (txHashInput.getAttribute("aria-invalid") === "true") {
    const validation = validateTxHash(txHashInput.value);
    setFieldError(validation.isValid ? "" : validation.message);
  }
});

loadWasmIndexer().then((indexer) => {
  wasmIndexer = indexer;
  setStatus(
    wasmIndexer
      ? "Ready for a transaction hash. Rust/Wasm indexer loaded."
      : "Ready for a transaction hash. Build app.wasm from rust-indexer to enable lookup.",
  );
});
