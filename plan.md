# DevPane Birlikte Geliştirme Planı

## Özet
DevPane’i adım adım sen yazacaksın; ben her adımda neyi neden yazdığını açıklayacağım, küçük kod önerileri vereceğim, ama sen anlamadan bir sonraki adıma geçmeyeceğiz. Backend/Rust kısmında editleri sen yapacaksın. Frontend/Tauri aşamasına gelince istersen bazı UI dosyalarını ben yazabilirim.

## Çalışma Şeklimiz
- Her oturumda tek küçük hedef seçeceğiz: örnek `validation.rs ekle`, `clap komutlarını kur`, `path resolution yaz`.
- Önce kavramı anlatacağım: hangi Rust konusu var, hangi dosya etkileniyor, başarı kriteri ne.
- Sonra sana yazılacak kodu parça parça önereceğim; sen editöre yazacaksın.
- Her parçada durup kontrol edeceğiz: “bu satır ne yapıyor?”, “borrow/reference neden böyle?”, “hata nereden dönüyor?”
- Ben repo okumayı, hata ayıklamayı, test/build çıktısını yorumlamayı ve gerektiğinde alternatif kod önermeyi üstleneceğim.
- Anlamadığın yerde plan durur; konuyu açarız, gerekirse mini örnekle devam ederiz.

## Milestone Sırası
1. **Config Parser Temizliği**
   - `main.rs` sadece `.dpane` dosyasını yükleyip okunabilir özet bassın.
   - `config.rs` veri modelinden sorumlu kalsın.
   - Kontrol: `cargo run` workspace adı, version ve pane listesini düzgün yazmalı.

2. **Validation**
   - `src/validation.rs` eklenecek.
   - İlk kurallar: `version == 1`, `name` boş değil, layout içindeki her `pane` id’si `panes` içinde var.
   - Sonra: split children boş olamaz, cwd çözümleme sonrası var mı kontrol edilir.
   - Kontrol: geçerli config başarıyla geçer; bozuk örneklerde anlaşılır hata verir.

3. **Path Resolution**
   - Workspace root ve pane cwd kuralları yazılacak.
   - `root` yoksa `.dpane` dosyasının klasörü kullanılacak.
   - Relative pane cwd, workspace root’a göre çözülecek.
   - Kontrol: Windows absolute path ve relative path örnekleri doğru sonuç üretir.

4. **CLI Komutları**
   - `clap` eklenecek.
   - Komutlar: `validate`, `inspect`, `run`.
   - İlk aşamada default path olarak `examples/webclient.dpane` kullanılabilir; sonra argüman zorunlu yapılır.
   - Kontrol: `cargo run -- validate examples/webclient.dpane` ve `inspect` ayrı davranmalı.

5. **Process Runner MVP**
   - `runner.rs` eklenecek.
   - `auto_start = true` olan pane’ler için command çalıştırılacak.
   - stdout/stderr pane id prefix’iyle basılacak.
   - Bu aşama gerçek terminal değil; sadece geçici CLI runner.
   - Kontrol: basit komutlarla process başlatma ve çıktı prefixleme çalışır.

6. **Tauri Öncesi Stabilizasyon**
   - Config, validation, path resolution ve runner sorumlulukları netleşecek.
   - Küçük testler eklenecek.
   - `cargo fmt`, `cargo check`, mümkünse `cargo clippy` temiz olacak.

7. **Tauri + Frontend**
   - Tauri kurulacak.
   - İlk UI: workspace title, pane listesi, layout placeholder.
   - Sonra tek xterm.js terminal pane.
   - En son recursive multi-pane layout, pane lifecycle ve Windows UX.

## İlk Başlayacağımız Adım
Sıradaki pratik ders/adım: **`validation.rs` ilk versiyonunu senin yazman**.

Başarı kriteri:
- `mod validation;` ile modül bağlanacak.
- `validate_config(config: &DevPaneConfig, config_path: &Path) -> anyhow::Result<()>` fonksiyonu olacak.
- Şimdilik `config_path` kullanılmasa bile imzada kalacak; ileride cwd validation için lazım.
- `version`, `name`, layout pane reference kontrolleri yapılacak.
- `main.rs`, config’i yükledikten sonra validation çağıracak.

## Test Planı
- `cargo check`
- `cargo run`
- Bilerek bozuk `.dpane` örnekleriyle elle deneme:
  - `version: 2`
  - boş `name`
  - layout içinde olmayan pane id
  - boş split children, bu kural eklendikten sonra

## Varsayımlar
- Rust backend kodunu sen yazacaksın; ben yönlendireceğim.
- Frontend/Tauri aşamasına kadar ben repo dosyalarını değiştirmeyeceğim.
- Her adım küçük olacak; anlamadan geçilmeyecek.
- DevPane cross-platform (Windows, macOS, Linux), hafif, React/Electron’suz ilerleyecek.
