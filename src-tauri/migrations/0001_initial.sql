-- Schéma initial — phase 2 de la migration vers Rust.
--
-- Remplace les deux versions du schéma Dexie (IndexedDB). Trois différences de
-- fond par rapport à ce que stockait l'ancienne base :
--
-- 1. Les montants sont des entiers de centimes, plus des flottants (défaut D5).
--    Un `REAL` SQLite est un f64 : conserver des montants dedans aurait
--    reconduit exactement le problème qu'on cherche à corriger.
--
-- 2. Les pièces comptables portent un `deleted_at` (défaut D4). Une facture
--    émise ne se supprime pas — conservation dix ans, art. L123-22 du code de
--    commerce. Seuls les brouillons peuvent disparaître pour de bon.
--
-- 3. Les numéros de pièce sont alloués par `document_sequences`, dans la même
--    transaction que l'insertion, et la contrainte d'unicité les protège
--    (défaut D3, art. 242 nonies A du CGI).
--
-- Les taux de TVA restent en REAL : ce sont des pourcentages (20, 5.5), pas des
-- montants, et ils ne s'additionnent jamais.

CREATE TABLE clients (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    company_name TEXT    NOT NULL,
    contact_name TEXT    NOT NULL DEFAULT '',
    email        TEXT    NOT NULL DEFAULT '',
    phone        TEXT    NOT NULL DEFAULT '',
    address      TEXT    NOT NULL DEFAULT '',
    created_at   TEXT    NOT NULL,
    deleted_at   TEXT
);

CREATE TABLE invoices (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    client_id          INTEGER REFERENCES clients (id) ON DELETE SET NULL,
    company_name       TEXT    NOT NULL,
    invoice_number     TEXT    NOT NULL UNIQUE,
    service_type       TEXT    NOT NULL,
    description        TEXT    NOT NULL DEFAULT '',
    amount_ht_cents    INTEGER NOT NULL,
    tva_rate           REAL    NOT NULL DEFAULT 0,
    amount_tva_cents   INTEGER NOT NULL,
    amount_total_cents INTEGER NOT NULL,
    date               TEXT    NOT NULL,
    status             TEXT    NOT NULL DEFAULT 'brouillon',
    payment_date       TEXT,
    payment_method     TEXT,
    deleted_at         TEXT
);

CREATE TABLE estimates (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    client_id          INTEGER REFERENCES clients (id) ON DELETE SET NULL,
    company_name       TEXT    NOT NULL,
    estimate_number    TEXT    NOT NULL UNIQUE,
    service_type       TEXT    NOT NULL,
    description        TEXT    NOT NULL DEFAULT '',
    amount_ht_cents    INTEGER NOT NULL,
    tva_rate           REAL    NOT NULL DEFAULT 0,
    amount_tva_cents   INTEGER NOT NULL,
    amount_total_cents INTEGER NOT NULL,
    date               TEXT    NOT NULL,
    status             TEXT    NOT NULL DEFAULT 'brouillon',
    deleted_at         TEXT
);

CREATE TABLE expenses (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    date           TEXT    NOT NULL,
    merchant       TEXT    NOT NULL,
    category       TEXT    NOT NULL DEFAULT 'Autre',
    amount_cents   INTEGER NOT NULL,
    description    TEXT    NOT NULL DEFAULT '',
    payment_method TEXT    NOT NULL DEFAULT 'carte',
    deleted_at     TEXT
);

-- Séquence de numérotation par type de pièce et par exercice.
--
-- `next_value` est le prochain numéro à attribuer. À la reprise d'une base
-- existante il est initialisé au plus grand numéro déjà émis plus un, et
-- jamais au nombre de lignes reprises : c'est précisément l'erreur que faisait
-- l'ancienne implémentation, et la reproduire ici réintroduirait D3.
CREATE TABLE document_sequences (
    kind       TEXT    NOT NULL,
    year       INTEGER NOT NULL,
    next_value INTEGER NOT NULL,
    PRIMARY KEY (kind, year)
);

-- Le livre des recettes et la déclaration URSSAF filtrent sur l'encaissement.
CREATE INDEX idx_invoices_payment_date ON invoices (payment_date);
CREATE INDEX idx_invoices_client       ON invoices (client_id);
CREATE INDEX idx_invoices_deleted      ON invoices (deleted_at);
CREATE INDEX idx_estimates_client      ON estimates (client_id);
CREATE INDEX idx_estimates_deleted     ON estimates (deleted_at);
CREATE INDEX idx_expenses_date         ON expenses (date);
CREATE INDEX idx_expenses_deleted      ON expenses (deleted_at);
