/**
 * Les calculs de période lisent l'année et le mois dans le fuseau local
 * (`getFullYear`, `getMonth`). Sans fuseau fixe, une facture du 31 décembre au
 * soir change d'exercice selon la machine qui exécute les tests.
 *
 * Le fuseau est donc épinglé sur celui des utilisateurs de l'application. La
 * réimplémentation Rust devra faire le même choix explicitement — c'est
 * précisément le genre de dépendance implicite qu'une migration révèle.
 */
process.env.TZ = 'Europe/Paris';
