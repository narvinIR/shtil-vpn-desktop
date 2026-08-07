<div align="center">
  <img src="src/assets/icon.png" alt="Штиль VPN" width="120" height="120">

  <h1>Штиль VPN — для компьютера</h1>

  <p><strong>Windows и macOS. Ключ из Telegram-бота, российские сайты открываются напрямую.</strong></p>

  <p><em>Shtil VPN for desktop — a Tauri + Vue client on the sing-box core (VLESS + Reality, TUN mode)
  for Windows and macOS. Subscription link from a Telegram bot, Russian sites stay on the direct
  route, signed over-the-air updates.</em></p>

  <p>
    <a href="https://github.com/narvinIR/shtil-vpn">Готовые файлы и установка · Downloads</a> ·
    <a href="https://github.com/narvinIR/shtil-vpn/blob/main/README.en.md">English</a> ·
    <a href="https://t.me/RealityVPNBot_bot">@RealityVPNBot_bot</a>
  </p>
</div>

Приложение подключается к VPN «Штиль» по ссылке-подписке из бота. Банки,
госуслуги и маркетплейсы при включённом VPN продолжают работать напрямую, мимо
туннеля — списки едут вместе с приложением и ниоткуда не скачиваются.

На телефоне и телевизоре тот же «Штиль» (Android, отдельный репозиторий) — вид
и цвета общие.

## Установка

Готовые файлы для всех устройств — на [витрине «Штиля»](https://github.com/narvinIR/shtil-vpn/releases/tag/apps)
(адреса там постоянные) или на странице [sub.ndvsdom54.ru/get](https://sub.ndvsdom54.ru/get),
которая сама подбирает файл под систему.

**Windows.** Скачайте `.exe` из [релизов](../../releases) и запустите. Windows
покажет «Windows защитила ваш компьютер»: нажмите **«Подробнее» → «Выполнить в
любом случае»**. Так ведёт себя любое приложение, раздаваемое мимо магазина: у
нас нет сертификата издателя, а чистый запуск с 2024 года не покупается даже за
деньги — Microsoft отменила мгновенное доверие для платных сертификатов.

**macOS.** Скачайте `.dmg` под свой процессор (Apple или Intel), перетащите
приложение в «Программы». Первый запуск система не разрешит: **Системные
настройки → Конфиденциальность и безопасность → «Открыть всё равно»**, затем
пароль администратора. Дальше открывается обычным двойным щелчком.

Ключ подписки — в боте [@RealityVPNBot_bot](https://t.me/RealityVPNBot_bot).

## Чем отличается от исходного проекта

Форк [xinggaoya/sing-box-windows](https://github.com/xinggaoya/sing-box-windows)
(MIT). Изменено:

- **Подписка берётся целиком.** Исходный клиент вытаскивал из конфига только
  серверы, а маршруты строил свои — на списках, которые скачиваются с GitHub.
  Из России эти адреса не отвечают, и ядро тогда не стартует вовсе. Теперь
  готовый конфиг применяется как есть; чужие подписки списком ссылок
  по-прежнему разбираются по узлам.
- **Вид «Штиля»** — палитра, знак и имя из нашего телефонного приложения.

## Разработка

```bash
pnpm install
pnpm kernel:fetch     # ядро sing-box кладётся в сборку
pnpm tauri dev        # запуск
pnpm tauri build      # сборка под текущую систему
```

Установщики для Windows и macOS собираются в GitHub Actions
(`.github/workflows/build.yml`): на своей машине собирается только та система,
на которой сидишь.

Проверки перед отправкой: `pnpm type-check`, `pnpm lint`,
`cargo test --manifest-path src-tauri/Cargo.toml`.

Стек: Tauri 2 + Vue 3 + TypeScript (интерфейс), Rust (служебная часть),
ядро [sing-box](https://github.com/SagerNet/sing-box) отдельным файлом.

## Лицензия

MIT — как у исходного проекта. Ядро sing-box распространяется под GPL-3.0 и
поставляется отдельным исполняемым файлом, в код не встраивается.
