# XGen Protocol — Príloha B – Ako sa XGen Protocol financuje bez toho, aby sa predal
> Status: done  
> Version: 0.1  
> Date: April 2026  
> Last edited: April 2026  
> Language: Slovak  
> Author: JozefN  
> License: BSL 1.1 (converts to GPL upon project handover)  

*Pozičný dokument o udržateľnosti pre spolupracovníkov a prispievateľov*

---

## Otázka, ktorá zabíja otvorené projekty

Väčšina open-source projektov nezlyháva kvôli zlému kódu. Zlyháva kvôli zlej ekonomike. Vzorec je deprimujúco známy: nadšený tím postaví niečo výnimočné, projekt získa prijatie, a potom nastane jedna z dvoch vecí. Buď projekt pomaly hynie — udržiavaný dobrovoľníkmi, ktorí sa jeden po druhom vyčerpávajú — alebo prijme firemné peniaze a ticho prestane slúžiť svojmu pôvodnému poslaniu.

XGen Protocol potrebuje tretiu cestu. A táto cesta musí byť navrhnutá vedome, od začiatku — nie improvizovaná neskôr, keď dôjdu peniaze.

---

## Lekcia z Blenderu

Najbližší organizačný model tomu, čím XGen potrebuje sa stať, je Blender Foundation. Nie preto, že Blender je komunikačný protokol — nie je — ale preto, že jeho história sa takmer presne mapuje na riziká a ambície XGen.

Stručne: Ton Roosendaal postavil Blender ako interný komerčný nástroj, vyčlenil ho do spoločnosti, vzal rizikový kapitál a sledoval, ako investori celý projekt zatvorili, keď praskla dot-com bublina v roku 2002. Zdrojový kód bol rukojemníkom veriteľov. Roosendaal spustil komunitnú fundraisingovú kampaň, za sedem týždňov nazbieral 100 000 €, kód vykúpil a vydal ho pod GPL licenciou. Potom postavil neziskovú nadáciu — holandskú Stichting — aby sa to nemohlo nikdy zopakovať.

O dvadsať rokov neskôr sa Blender používa v hollywoodskych produkciách, herných štúdiách a na univerzitách po celom svete. Nikdy neprijal ani cent investičného kapitálu. Nikdy neodpovedal predstavenstvu akcionárov. Nikdy nebol odkúpený.

To je plán.

Lekcie, ktoré XGen berie z Blenderovej cesty, sú konkrétne:

**Nikdy neprijmite investičný kapitál.** V momente, keď máte investorov, máte zainteresované strany, ktorých záujmy sa rozchádzajú so záujmami vašich používateľov. Toto nie je hypotetické — je to mechanizmus za každým platformovým zradením za posledných dvadsať rokov. Skype, WhatsApp, Instagram — všetky začali s úprimnými misiami. Všetky nakoniec slúžili svojim nadobúdateľom.

**Zaregistrujte sa ako nezisková organizácia od prvého dňa.** Nie neskôr, keď to bude nevyhnutné. Od začiatku. Právna štruktúra musí urobiť misiu štrukturálne trvalou, nielen kultúrne udržiavanou. Kultúra sa mení. Právne štruktúry sa menia ťažšie.

**Diverzifikujte zdroje príjmov.** Blenderov súčasný model je nebezpečne závislý od darov — jediný prúd, ktorý dominuje jeho príjmom. Aj v dobrých rokoch s veľkými donorskými kampaňami vykazujú straty. XGen sa tomu musí vyhnúť od dizajnu: žiadny jednotlivý príjmový prúd by nemal presiahnuť zhruba 30-40 % celkových príjmov.

---

## Päť príjmových prúdov

Model udržateľnosti XGen je postavený na piatich samostatných prúdoch. Každý je nezávislý. Každý slúži inému okruhu zainteresovaných. Spolu vytvárajú odolnosť — ak sa niektorý prúd zúži, ostatné ho udržia.

---

### Prúd 1 — Dobrovoľné dary

Jednotliví používatelia a vývojári, ktorí veria v projekt, prispievajú čím môžu. Toto je primárny prúd Blenderovho modelu a funguje — ale iba v dostatočnom meradle a iba keď projekt preukázal skutočnú hodnotu.

Úprimné zhodnotenie tohto prúdu: je najdemokratickejší a najmenej spoľahlivý. Blenderova kampaň „Join the 2%" je priznaním, že veľká väčšina používateľov nikdy neplatí. Funguje, keď máte milióny používateľov. Je nedostatočný ako primárny prúd počas raného rastu.

Pre XGen slúžia dobrovoľné dary inému účelu nad rámec príjmov: sú signálom. Veľká, aktívna základňa darcov demonštruje vlastníctvo komunity a nezávislosť inštitucionálnym partnerom, grantovým komisiám a potenciálnym firemným členom. Počet je rovnako dôležitý ako výška.

---

### Prúd 2 — Členstvo vo firemnom vývojovom fonde

Spoločnosti, ktoré stavajú na protokole XGen alebo z neho priamo profitujú, platia ročný členský príspevok Nadácii. Na oplátku dostávajú včasný prístup k návrhom špecifikácií, účasť v pracovných skupinách a uznanie ako členov Nadácie.

Blender to robí úspešne — významné štúdiá a spoločnosti herných enginov prispievajú ročne. Model funguje, pretože tieto spoločnosti majú skutočný záujem na pokračujúcom zdraví protokolu. Banka, ktorá nasadí sieť XGen Tier 3 pre internú komunikáciu, nechce, aby protokol stagnoval.

Kritické pravidlo správy: **žiadny jednotlivý firemný člen nesmie prispieť viac ako 20 % z celkového objemu tohto prúdu.** Toto nie je zdvorilostný limit. Je to tvrdé štrukturálne pravidlo zakotvené v stanovách Nadácie. V momente, keď jedna korporácia prispeje dosť na to, aby sa cítila oprávnená ovplyvňovať rozhodnutia o cestovnej mape, je ohrozená nezávislosť protokolu. Strop bráni vzniku tejto dynamiky.

---

### Prúd 3 — Poplatky za certifikáciu modulov

Tento prúd neexistuje pre Blender. Neexistuje pre Matrix, Signal ani žiadny porovnateľný otvorený projekt. Existuje pre XGen vďaka architekture vrstvenej autentifikácie XGen — a je potenciálne najvýznamnejším a najstabilnejším prúdom z piatich.

Mechanizmus je nasledovný: organizácie, ktoré potrebujú oficiálne certifikovaný autentifikačný modul — vládna agentúra, nemocnica, banka, právnická firma — nemôžu jednoducho jeden postaviť a vyhlásiť ho za vyhovujúci. Potrebujú, aby Nadácia certifikovala, že ich modul správne implementuje príslušný vrstevný štandard a spĺňa súvisiace regulačné požiadavky (eIDAS pre európske inštitúcie, NIST IAL pre US federálne, ISO 29115 medzinárodne).

Nekupujú protokol. Protokol je zadarmo. Kupujú pečiatku súladu — auditovanú, zdokumentovanú, Nadáciou vydanú certifikáciu, že ich modul spĺňa štandard. Táto certifikácia má reálnu peňažnú hodnotu pre regulované odvetvia, pretože bez nej právne a compliance tímy nemôžu schváliť nasadenie.

Toto je model príjmov, ktorý priamo vychádza z dizajnovej filozofie XGen. Architektúra vrstvenej autentifikácie bola navrhnutá tak, aby slúžila inštitucionálnym potrebám. Certifikačné poplatky sú prirodzeným ekonomickým dôsledkom tohto dizajnu. Inštitúcia, ktorá platí za certifikáciu, zároveň financuje Nadáciu a potvrdzuje podnikové kredibility protokolu.

---

### Prúd 4 — Hosťovaná referenčná infraštruktúra

Protokol XGen je plne decentralizovaný. Ktokoľvek môže prevádzkovať vlastný uzol. Žiadna inštitúcia nie je povinná používať infraštruktúru hosťovanú Nadáciou. To je princíp dizajnu, nie komerčné obmedzenie.

Nie každá organizácia však chce prevádzkovať vlastnú infraštruktúru. Prevádzka referenčného uzla, udržiavanie služby bootstrappingu identity, prevádzkovanie sandbox prostredia pre vývojárov — to vyžaduje technickú kapacitu, ktorú mnohé organizácie radšej zaplatia, než aby ju budovali interne.

Nadácia prevádzkuje túto hosťovanú infraštruktúru ako voliteľnú platenú službu. Organizácie, ktoré chcú spravovaný prístup, zaň platia. Organizácie, ktoré chcú plnú kontrolu, prevádzkujú vlastnú. Protokol funguje identicky v oboch prípadoch.

Analógiou je Red Hat a Linux. Linux bol zadarmo. Red Hat účtoval za podnikovú podporu, certifikované konfigurácie a spravované služby okolo bezplatného Linuxu. Red Hat postavil miliardový biznis bez toho, aby vlastnil Linux a bez toho, aby kompromitoval nezávislosť Linuxu. Prúd hosťovanej infraštruktúry aplikuje rovnakú logiku v menšom a cielenejšom meradle.

---

### Prúd 5 — Granty

Európska únia aktívne financuje prácu na otvorených protokoloch a digitálnej infraštruktúre už niekoľko rokov. Program EU Horizon, iniciatíva NGI (Next Generation Internet) a rôzne národné programy digitálnej suverenity spoločne nasmerovali stovky miliónov eur práve na typ projektu, ktorý XGen predstavuje.

XGen je nezvyčajne silným kandidátom na grant z niekoľkých konkrétnych dôvodov. Jeho architektúra je natívne GDPR by design — identita je overená, ale kontrolovaná používateľom a protokolom, nie zbieraná centrálnou platformou. Jeho autentifikačný vrstevný systém je kompatibilný s eIDAS, štandardom EÚ pre elektronickú identifikáciu. Jeho model inštitucionálnej nezávislosti je v súlade s cieľmi európskej digitálnej suverenity. Jeho federovaná architektúra znižuje závislosť od infraštruktúry so sídlom v USA.

Grantové financovanie nie je pasívny príjem. Vyžaduje si dedikovanú schopnosť písania grantov v tíme — niekoho, kto rozumie procesom podávania žiadostí, hovorí jazykom týchto financujúcich orgánov a dokáže udržiavať reportingové požiadavky, ktoré sprevádzajú verejné financovanie. Toto je špecifická zručnosť, ktorá musí byť zastúpená v prvých pracovných miestach Nadácie. Nie je to niečo, čo možno improvizovať, keď sa objaví grantová príležitosť.

Strategická hodnota grantov presahuje peniaze. Úspešný grant EU Horizon je formou inštitucionálnej validácie. Signalizuje vládam, regulovaným odvetviam a podnikovým adoptérom, že XGen bol vyhodnotený a uznán za dôveryhodný serióznym financujúcim orgánom. Tento signál má hodnotu ďaleko presahujúcu výšku samotného grantu.

---

## Prečo päť prúdov a nie jeden

V raných štádiách projektov je lákavé nájsť jeden veľký zdroj príjmov a sústrediť sa naň. Firemné sponzorstvo, alebo grantová nadácia, alebo veľký donor. Jednoduchosť je príťažlivá.

Problém je koncentračné riziko. Každý jednotlivý prúd môže zmiznúť. Veľký firemný člen je odkúpený a nová materská spoločnosť sa stiahne. Grantový program zmení priority. Donorská kampaň zaostane. Ak tento prúd predstavuje 80 % príjmov, projekt je v kríze.

Model piatich prúdov XGen je navrhnutý tak, aby zlyhanie žiadneho jednotlivého prúdu neohrozilo prežitie projektu. Ak sa firemné členstvá v recesii zúžia, poplatky za certifikáciu modulov z regulovaných odvetví (ktoré sú proticyklické — compliance požiadavky v recesiách nezmiznú) udržia model stabilným. Ak sa grantové financovanie vyschne, príjmy z hosťovanej infraštruktúry pokračujú. Ak sa veľký firemný člen stiahne, strop 20 % znamená, že ich odchod nerozloží prúd.

Diverzita príjmov je diverzita nezávislosti. Každý prúd slúži inému okruhu zainteresovaných s rôznymi stimulmi. Žiadny jednotlivý okruh nemôže protokol držať ako rukojemníka.

---

## Pravidlo, ktoré drží všetko pohromade

Každé rozhodnutie o správe a udržateľnosti v XGen sa vracia k jednému princípu: **protokol musí byť štrukturálne neschopný byť zachytený.**

Nie len kultúrne odolný voči zachyteniu. Nie len vedený ľuďmi s dobrými úmyslami. Štrukturálne neschopný — čo znamená, že aj keby sa ľudia zmenili, aj keby sa stimuly posunuli, právna a finančná architektúra robí zradu nemožnou alebo prinajmenšom neúnosne ťažkou.

Nezisková štruktúra robí akvizíciu nemožnou bez rozpustenia Nadácie. Diverzifikácia príjmov robí páku akéhokoľvek jednotlivého financovateľa nedostatočnou na diktovanie podmienok. Firemný strop 20 % bráni neformálnemu vplyvu prostredníctvom finančnej dominancie. Licencia otvoreného protokolu znamená, že kód nemožno uzamknúť, aj keby bola samotná Nadácia nejako kompromitovaná.

Toto je to, čo „postavené generáciou, ktorá sledovala zničenie každej dobrej platformy" v praxi skutočne znamená. Nie nostalgia. Nie rétorika. Konkrétne štrukturálne rozhodnutia prijaté vopred, aby sa zabránilo konkrétnym vzorcom zlyhania, ktoré boli pozorované tridsať rokov.

---

## Čo to vyžaduje od prvého tímu

Model udržateľnosti opísaný tu nefunguje sám od seba. Vyžaduje konkrétne schopnosti v zakladajúcom tíme, ktoré musia byť naplánované:

Schopnosť písania grantov nie je voliteľná — Prúd 5 je potenciálne transformačný v prvých rokoch, ale iba ak niekto dokáže žiadosti skutočne napísať. Toto je špecializovaná zručnosť, nie všeobecná kompetencia.

Právna a compliance schopnosť je potrebná na operacionalizáciu Prúdu 3 — poplatky za certifikáciu modulov vyžadujú, aby Nadácia skutočne vykonávala certifikácie, čo si vyžaduje zdokumentované procesy, právne preskúmanie a technickú audítorskú kapacitu.

Schopnosť správy komunity udržiava Prúd 1 — dobrovoľné dary sa škálujú so zapojením komunity a zapojenie komunity je práca.

Toto nie sú roly, ktoré treba obsadiť neskôr. Sú to roly, ktoré treba plánovať pred napísaním prvého riadku špecifikácie protokolu.

---

*XGen Protocol — Pozičný dokument o udržateľnosti*
*Apríl 2026*
