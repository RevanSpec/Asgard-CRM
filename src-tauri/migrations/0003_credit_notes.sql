-- Avoirs.
--
-- Une facture émise ne se supprime ni ne se modifie : elle s'annule ou se
-- corrige par un avoir (art. 242 nonies A du CGI). L'application le disait déjà
-- dans ses messages de suppression, sans offrir le moyen de le faire.
--
-- L'avoir est une pièce à part entière : sa propre série de numéros (AVO-), sa
-- date, son motif, et une référence obligatoire à la facture qu'il corrige. Il
-- peut être partiel — un geste commercial après coup — d'où des montants
-- saisis plutôt que recopiés.
--
-- Les montants sont stockés **positifs** : c'est ainsi que la pièce se lit
-- (« avoir de 120 € »). Le signe s'applique au moment de l'agrégation, où
-- l'avoir devient une recette négative.
--
-- `refunded_on` porte la date du remboursement effectif. Elle joue le rôle du
-- règlement d'une facture : en comptabilité de trésorerie, l'avoir ne diminue
-- les recettes que lorsque l'argent repart.

CREATE TABLE credit_notes (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    invoice_id         INTEGER NOT NULL REFERENCES invoices (id),
    client_id          INTEGER REFERENCES clients (id) ON DELETE SET NULL,
    company_name       TEXT    NOT NULL,
    credit_number      TEXT    NOT NULL UNIQUE,
    service_type       TEXT    NOT NULL,
    description        TEXT    NOT NULL DEFAULT '',
    amount_ht_cents    INTEGER NOT NULL,
    tva_rate           REAL    NOT NULL DEFAULT 0,
    amount_tva_cents   INTEGER NOT NULL,
    amount_total_cents INTEGER NOT NULL,
    date               TEXT    NOT NULL,
    refunded_on        TEXT,
    deleted_at         TEXT
);

CREATE INDEX credit_notes_invoice ON credit_notes (invoice_id);
