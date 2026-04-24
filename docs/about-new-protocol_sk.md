# Prečo musí byť XGen Protocol vlastným protokolom — a nie stavbou na cudzom
> Status: temporaly done
> Version: 0.1
> Date: April 2026
> Last edited: April 2026
> Language: Slovak
> Author: JozefN
> License: BSL 1.1 (converts to GPL upon project handover)

*Pozičný dokument pre spolupracovníkov a prispievateľov*

---

## Otázka, ktorú stojí za to položiť úprimne

Keď niekto navrhne budovať nový komunikačný protokol od základov, zjavná námietka je: *prečo nevyužiť to, čo už existuje?* Matrix je otvorený. Signal je rešpektovaný. XMPP existuje desaťročia. ActivityPub pohára Fediverse. Prečo pridávať ďalšiu vrstvu do už tak preplneného priestoru?

Je to spravodlivá otázka. A zaslúži si presnú odpoveď — nie odmietnutie.

---

## Čo v skutočnosti znamená „stavať na existujúcom protokole"

Keď staviate *na vrchole* existujúceho protokolu, zdedíte tri veci: jeho silné stránky, jeho obmedzenia a jeho **dizajnové predpoklady**. Tretia vec je tá, ktorá projekty zabíja.

Každý protokol v sebe zakóduje určitý svetonázor. Rozhodnutia zapečené do jeho základov — čo znamená identita, ako sa modeluje dôvera, kto riadi zmeny, ktoré prípady použitia boli uprednostnené — urobili konkrétni ľudia, v konkrétnom momente, s konkrétnymi cieľmi. Tieto rozhodnutia nezostávajú navždy neutrálne. Hromadia sa do gravitačného poľa, ktoré ťahá každý projekt postavený na ich vrchole smerom k tomu istému centru.

XGen Protocol má špecifickú, neodvolateľnú dizajnovú požiadavku: **overená identita ako prvotriedny primitív protokolu.** Nie plugin. Nie voliteľná vrstva. Nie niečo, čo sa pridá dodatočne. Celý model dôvery závisí od toho, aby bola zakladaná — zapečená do každej správy, každej relácie, každej komunitnej interakcie na úrovni protokolu.

Žiadny existujúci protokol nebol postavený s týmto predpokladom. Každý existujúci protokol bol postavený s opačným predpokladom: že identita je niekoho iného problém.

---

## Existujúci kandidáti — a prečo každý z nich nestačí

### Matrix / Element

Matrix je najvážnejší konkurent v priestore open-protokolov pre komunitnú komunikáciu a zaslúži si úprimné zhodnotenie.

**Čo robí správne:** Skutočne federovaný. Otvorená špecifikácia. Aktívna vývojárska komunita. Mosty k iným platformám. Rozumná správa cez Matrix.org Foundation.

**Kde štruktúrne zlyháva pre účely XGen:**

Identitný model Matrixu je pseudonymný z dizajnu. Matrix ID (`@user:server.tld`) je trvalý identifikátor viazaný na homeserver — ale neexistuje žiadna požiadavka, mechanizmus ani zámer, aby toto ID zodpovedalo overenému skutočnému človeku. Protokol bol explicitne navrhnutý ako kompatibilný s anonymitou, čo je architektonická voľba priamo v rozpore s pilierom overenej identity XGen.

Čo je ešte závažnejšie: Matrix nesie značný technický dlh z raných architektonických rozhodnutí. Algoritmus rozlišovania stavu miestnosti, model udalostného DAG a federačný protokol majú known problémy s výkonom pri škálovaní, okolo ktorých jadro tímu roky pracuje. Stavať identitný a dôverový model XGen na týchto obmedzeniach by znamenalo zdediť problémy, ktoré XGen zdediť nemusí.

Matrix má tiež jednu dominantnú riadiacu entitu — Matrix.org — čo vytvára de facto centralizačný bod aj vo formálne federovanom systéme. Pilier inštitucionálnej nezávislosti XGen je nezlučiteľný so zakladajúcou závislosťou na rozhodnutiach o cestovnej mape inej organizácie.

### Signal Protocol

Signal Protocol je pravdepodobne zlatý štandard pre end-to-end šifrované správy. Je dobre navrhnutý, odskúšaný v praxi a bol prijatý WhatsAppom, Google Messages a ďalšími.

**Základný nesúlad:** Signal je point-to-point a maloskupinový správový protokol optimalizovaný pre súkromie a popierateľnosť. Bol explicitne navrhnutý pre scenáre, kde chcú zúčastnené strany komunikovať bez toho, aby tretia strana dokázala overiť, kto čo povedal. To je presne opačný dizajnový cieľ od XGen.

Mechanizmy Double Ratchet a Sealed Sender Signalu sú elegantné práve preto, lebo minimalizujú expozíciu identity. XGen potrebuje protokol, kde expozícia identity (príslušným stranám, na príslušných úrovniach dôvery) je vlastnosť, nie chyba. Nemožno postaviť model dôverových vrstiev XGen na protokole, ktorý bol navrhnutý tak, aby boli dôverové vrstvy technicky nemožné.

### XMPP

XMPP (Extensible Messaging and Presence Protocol) existuje od roku 1999 a zostáva v aktívnom použití. Je skutočne rozšíriteľný cez XEPs (XMPP Extension Protocols).

**Úprimné zhodnotenie:** Rozšíriteľnosť XMPP je zároveň jeho slabosťou. Jadro protokolu je minimálne z dizajnu a prakticky všetko užitočné — hlas/video cez Jingle, viacužívateľský chat cez MUC, prenos súborov, push notifikácie — je implementované cez rozšírenia s nekonzistentným prijatím naprieč klientmi a servermi. Výsledkom je v praxi fragmentovaný ekosystém, kde „XMPP kompatibilný" zriedkakedy znamená „plne interoperabilný."

Ešte zásadnejšie: identitný model XMPP je federovaný pomocou JID (Jabber IDs), ktoré nenesú žiadne inherentné tvrdenie o dôvere. Naroubovanie modulárneho autentifikačného vrstveného systému XGen na XMPP by vyžadovalo buď navrhnúť paralelnú identitu vrstvu, ktorá efektívne nahradí jadro XMPP — v tomto bode už zmysluplne „nestaviate na XMPP" — alebo natrvalo prijať jeho obmedzenia.

### ActivityPub

ActivityPub pohára Mastodon, PeerTube, Pixelfed a širší Fediverse. Je to štandard W3C pre federované sociálne siete.

**Nesúlad:** ActivityPub je protokol pre sociálny obsah — modeluje aktérov, objekty a aktivity (príspevky, lajky, sledovania). Nebol navrhnutý pre komunikáciu v reálnom čase. Nemá natívne primitívy pre hlas ani video, žiadny model relácie, žiadny koncept dôverových vrstiev a žiadny mechanizmus pre druh organizačnej štruktúry komunity (servery, kanály, roly, oprávnenia), ktorú XGen potrebuje. Prispôsobenie ActivityPub pre rozsah XGen by nebolo stavanie na ActivityPub — bolo by to budovanie nového protokolu, ktorý náhodou používa syntax ActivityPub.

---

## Hlbší architektonický dôvod

Existuje štrukturálny argument, ktorý presahuje obmedzenia jednotlivých protokolov.

Požiadavka XGen na overenú identitu nie je funkcia, ktorú možno pridať na vrchol protokolu. Je to **model dôvery** — a modely dôvery musia byť konzistentné od konca po koniec, alebo sú zbytočné. Protokol, kde je overenie identity voliteľné, alebo kde existuje iba na aplikačnej vrstve, alebo kde ho možno obísť pripojením k neoverenému uzlu, neposkytuje žiadnu zmysluplnú záruku.

Aby modulárny autentifikačný systém XGen fungoval — aby správa niesla kryptograficky overiteľné tvrdenie, že odosielateľ je Tier 2 (overená profesionálna identita) alebo Tier 3 (autentifikovaný firemným PKI) — toto tvrdenie musí byť vložené do formátu správy protokolu, overené smerovacou vrstvou protokolu a vynucované federačnými pravidlami protokolu. Nemôže to byť dodatočná anotácia na protokole, ktorý bol navrhnutý bez nej.

Stavanie toho na existujúcom protokole znamená buď:

1. **Obmedzenie XGen** na prácu v rámci dátového modelu a obmedzení existujúceho protokolu — čo zásadne kompromituje dizajn.
2. **Forkovanie existujúceho protokolu** natoľko výrazne, že sa udržiava fork, nie kompatibilné rozšírenie — čo znamená zdediť všetku zložitosť bez žiadneho ekosystémového benefitu.
3. **Prijatie rozdelenej architektúry**, kde kritická identitná vrstva žije mimo protokolu — čo znamená, že ju môže odstrániť, obísť alebo ignorovať akákoľvek implementácia, ktorá sa tak rozhodne.

Žiadny z týchto výsledkov nie je prijateľný pre protokol, ktorého celá hodnotová propozícia spočíva na štrukturálnej dôveryhodnosti.

---

## Historický precedens je jasný

Toto nie je nezvyčajná pozícia. Každý veľký protokol, ktorý sa stal základnou infraštruktúrou, začal tým, že rozpoznal, že existujúce nástroje nie sú primerané novým požiadavkám — a postavil od základov, nie adaptoval.

TCP/IP nestavalo na existujúcom protokole okruhovo prepínanej telefónnej siete. HTTP nestavalo na FTP. SMTP sa nepokúšalo prispôsobiť existujúcu metaforu papierovej pošty. Identifikovali, čo existujúci model pre ich prípad použitia štrukturálne zle robí, a postavili nový základ.

Signal Protocol sám nestavalo na šifrovacích rozšíreniach XMPP. Identifikoval existujúci prístup ako architektonicky nedostatočný pre jeho model hrozby a navrhol niečo nové.

Otázka nikdy nebola *„existuje niečo?"* Otázka vždy bola *„zodpovedá to, čo existuje, základným dizajnovým požiadavkám?"* Keď je odpoveď nie, správnym krokom je budovať — nie kompromitovať požiadavky, aby sa zmestili do dostupných nástrojov.

---

## Čo to v praxi znamená

Budovanie XGen ako nezávislého protokolu znamená:

**Viac práce na začiatku.** Neexistuje skratka cez existujúci ekosystém. Špecifikácia protokolu, referenčná implementácia a vývojárske nástroje musia byť postavené od základov. To je úprimná cena.

**Žiadne zdedené obmedzenia.** Protokol možno navrhnúť okolo skutočných požiadaviek XGen — overená identita, modulárne dôverové vrstvy, komunitná komunikácia v reálnom čase pri škálovaní — bez toho, aby bola ktorákoľvek z nich kompromitovaná, aby sa zmestila do obmedzení zdedených z iného projektu s inými cieľmi.

**Žiadna závislosť od správy.** Pilier inštitucionálnej nezávislosti XGen vyžaduje, aby žiadna inštitúcia prijímajúca XGen nemusela závisieť od cestovnej mapy inej organizácie, schválenia normalizačného orgánu alebo politických rozhodnutí. Táto nezávislosť je štrukturálne nemožná, ak je základom XGen protokol inej organizácie.

**Plná kompatibilita tam, kde na nej záleží.** XGen môže definovať mosty a vrstvy interoperability s existujúcimi protokolmi — Matrix, Signal, XMPP — ako vedomú voľbu kompatibility. Rozdiel je v tom, že tieto mosty sú voľbou XGen, nie obmedzením XGen. Jadro protokolu zostáva nekompromitované.

---

## Úprimné zhrnutie

Stavanie na existujúcom protokole je správnou voľbou, keď sú vaše požiadavky kompatibilné s dizajnovými predpokladmi tohto protokolu. Urýchľuje vývoj, poskytuje existujúci ekosystém a zabraňuje opätovnému vynájdeniu vyriešených problémov.

Požiadavky XGen nie sú kompatibilné s dizajnovými predpokladmi žiadneho existujúceho protokolu. Základná požiadavka — overená identita ako prvotriedny, nezfalšovateľný, kryptograficky vynucovaný primitív protokolu — bola zámerné vylúčená z každého existujúceho otvorené komunikačného protokolu. To nebola opomenutie. Bola to filozofická voľba týchto projektov. XGen robí opačnú filozofickú voľbu.

Keď je základ pre budovu, ktorú potrebujete postaviť, nesprávny, neprispôsobujete budovu. Položíte nový základ.

---

*XGen Protocol — Pozičný dokument*
*Apríl 2026*
