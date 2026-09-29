-- Réglages de l'entreprise, et état du premier lancement.
--
-- Les réglages vivaient dans le `localStorage` de la WebView. Aucune sauvegarde
-- automatique ne les couvrait : la copie quotidienne est un `VACUUM INTO` de ce
-- fichier, et l'identité, le logo, les gabarits d'e-mail et la configuration
-- SMTP étaient ailleurs (défaut D13). Un profil de WebView nettoyé les effaçait
-- sans que rien ne le signale, et seule l'exportation JSON manuelle les
-- embarquait.
--
-- La base fait désormais foi. Le `localStorage` reste un cache : l'interface lit
-- les réglages pendant le rendu, donc sans pouvoir attendre une réponse de
-- l'hôte. La règle est simple et tient en une phrase — au démarrage, ce que dit
-- la base écrase le cache ; une base qui n'a rien reçoit ce que le cache
-- contient, ce qui reprend les réglages des installations existantes.
--
-- Une table clé-valeur plutôt que des colonnes : les réglages sont un objet de
-- l'interface, avec ses champs facultatifs et ses ajouts futurs, et
-- `#[serde(default)]` relit déjà un JSON auquel il manque des champs. Les
-- découper en colonnes obligerait à une migration par réglage ajouté.

CREATE TABLE app_settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
