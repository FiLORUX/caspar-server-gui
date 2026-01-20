CasparCG – ladda samma HTML två gånger (key-mask + fill)
A) Kör från URL (rekommenderas för ?1&mode=...)

CasparCG kan spela URL:er via HTML-producern.

Exempel (Channel 1, layer 20 = fill, layer 19 = key):

CG 1-20 ADD 0 "http://127.0.0.1:8080/tsg_keyfill_slate.html?1&mode=fill" 1
CG 1-19 ADD 0 "http://127.0.0.1:8080/tsg_keyfill_slate.html?1&mode=key"  1
MIXER 1-19 KEYER 1


MIXER … KEYER 1 gör att layer 19 används som key för layer 20 och inte renderas som bild.

Om din Caspar-version kräver [HTML]-prefix för vissa URL-fall: använd PLAY 1-20 [HTML] "http://..." istället.

B) “Ren” key+fill utan key-mask-tricket (om du inte behöver olika text)

Då räcker transparent body/html och CasparCG genererar key+fill automatiskt till ett decklink-par.

3) vMix (samma idé)

Lägg två Browser Inputs:

...tsg_keyfill_slate.html?1&mode=fill

...tsg_keyfill_slate.html?1&mode=key

Använd dem som fill/key-par i din output-mapping.

Om du vill att ?1 även ska styra en stor “ID-platta” i ett separat hörn (för snabb kabel-identifiering i multiview), säg var du vill ha den (t.ex. övre vänster på fill, övre höger på key) så justerar jag layouten utan att förändra key-mask-logiken.
