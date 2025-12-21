# Attendance Bot

A simple Discord bot to track study attendance:

-   When a member joins configured voice channel(s), they are marked as attended and a confirmation message is posted to a text channel.
-   Anyone can run `/attendance` to see today's attendance list. Admins (Manage Server) can run `/attendance_reset`.

## Setup

1. Create a bot in the Discord Developer Portal and add it to your server.
2. Enable Privileged Gateway Intents for the bot:
    - Server Members Intent
    - Presence Intent (not required)
    - Message Content Intent (not required)
3. Copy the example env and fill in your IDs:

```fish
cp .env.example .env
# Edit .env and set DISCORD_TOKEN, APPLICATION_ID, GUILD_ID, TEXT_CHANNEL_ID, VOICE_CHANNEL_IDS
```

-   `VOICE_CHANNEL_IDS` should be a comma-separated list of the voice channel IDs to track.
-   `TIMEZONE` affects the date boundary for daily attendance (default UTC).

## Install and run (fish shell)

This project uses TypeScript and discord.js. From this folder:

```fish
# install deps (choose one)
pnpm install; and echo ok
# or
npm install; and echo ok

# run in dev mode
pnpm dev; or npm run dev

# or build and run
pnpm build; and node dist/index.js
```

If you use `pnpm`, ensure it's installed (`curl -fsSL https://get.pnpm.io/install.sh | sh -`). Node 18+ required.

## Slash commands

Commands are auto-registered in your guild on startup:

-   `/attendance` — show today's attendance
-   `/attendance_reset` — reset today's list (requires Manage Server permission)

If you change command definitions, restart the bot to re-register.

## Data storage

Attendance is saved under `data/attendance-YYYY-MM-DD.json` for each day (based on TIMEZONE). Commit or ignore these files as you prefer.

## Notes

-   The bot only marks attendance when users join one of the configured voice channels.
-   Duplicate joins are ignored for the same day; name updates are reflected.
-   The confirmation message is posted to the configured text channel.
