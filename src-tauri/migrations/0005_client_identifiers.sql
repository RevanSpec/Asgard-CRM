-- Identifiants du client et nature de l'opération.
--
-- Le décret n° 2022-1299 ajoute quatre mentions obligatoires sur les factures,
-- applicables avec l'obligation de facturation électronique : le numéro SIREN
-- du client, l'adresse de livraison lorsqu'elle diffère de l'adresse de
-- facturation, la nature de l'opération — livraisons de biens, prestations de
-- services, ou les deux — et, le cas échéant, l'option pour le paiement de la
-- TVA d'après les débits.
--
-- Les trois premières se saisissent ; la quatrième est un réglage de l'émetteur
-- et vit avec les autres réglages. Le format Factur-X lui-même n'est pas
-- produit ici : ce qui compte est que les données existent le jour venu, sans
-- avoir à les redemander client par client.
--
-- **La nature est stockée, pas déduite.** Elle se devine du type d'activité —
-- une vente de marchandises est une livraison de biens, une prestation en est
-- une — mais une pièce émise ne se modifie pas : changer la règle de déduction
-- ne doit pas changer ce qu'une facture déjà envoyée affirme. Même raisonnement
-- que l'échéance, migration 0002. Le cas mixte, lui, ne se déduit d'aucun type.
--
-- Les pièces antérieures gardent une nature nulle, et le PDF la déduit alors de
-- leur type d'activité : c'est ce qu'elles disaient déjà implicitement.

ALTER TABLE clients ADD COLUMN siren            TEXT NOT NULL DEFAULT '';
ALTER TABLE clients ADD COLUMN vat_number       TEXT NOT NULL DEFAULT '';
ALTER TABLE clients ADD COLUMN delivery_address TEXT NOT NULL DEFAULT '';

ALTER TABLE invoices  ADD COLUMN operation_kind TEXT;
ALTER TABLE estimates ADD COLUMN operation_kind TEXT;
