-- Échéance de règlement.
--
-- L'article L441-9 du code de commerce impose de faire figurer sur la facture
-- la date à laquelle le règlement doit intervenir. Le pied de page annonçait
-- « sous 30 jours » sans jamais nommer de date.
--
-- L'échéance est **stockée** plutôt que recalculée à l'affichage : une facture
-- émise ne se modifie pas, et changer le délai par défaut dans les réglages ne
-- doit pas déplacer l'échéance des factures déjà envoyées.
--
-- Les factures antérieures gardent une échéance nulle : rien ne permet de
-- deviner le délai qui leur avait été annoncé, et inventer une date sur une
-- pièce déjà émise serait pire que de n'en imprimer aucune.

ALTER TABLE invoices ADD COLUMN due_date TEXT;
