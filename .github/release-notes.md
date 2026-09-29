## Installation

Téléchargez l'installeur ci-dessous — `Asgard CRM_<version>_x64-setup.exe`,
environ 3,7 Mo — et lancez-le.

> ⚠️ **Windows affichera un avertissement** au premier lancement : « Windows a
> protégé votre ordinateur ». C'est attendu — le programme n'est pas signé, un
> certificat de signature coûtant plusieurs centaines d'euros par an pour un
> projet personnel. Cliquez sur **Informations complémentaires**, puis sur
> **Exécuter quand même**.

L'installeur télécharge **WebView2** s'il manque à la machine. Il est présent
d'origine depuis Windows 10 21H2 ; une machine plus ancienne **et hors ligne**
ne pourra donc pas terminer l'installation.

Vos données restent chez vous, dans `%APPDATA%\com.asgard.crm` : la base SQLite,
ses copies quotidiennes, et rien d'autre. Aucune n'est envoyée nulle part.
