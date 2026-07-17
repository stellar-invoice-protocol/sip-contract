# Soroban Storage Models

Uses a combination of Persistent storage (for invoice details) and Instance storage (for platform stats).

Invoice references use persistent `InvoiceReference(u64)` entries keyed by invoice ID. They are stored
separately from invoice records so the serialized `Invoice` layout remains backward-compatible. A
reference entry is written at most once and contains a non-empty Soroban `String` of at most 64 bytes.
