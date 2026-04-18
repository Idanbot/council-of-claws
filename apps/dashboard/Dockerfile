FROM node:24-alpine AS build

WORKDIR /app
COPY package.json package-lock.json svelte.config.js vite.config.ts tsconfig.json tailwind.config.ts ./
COPY src ./src
RUN npm ci && npm run build

FROM node:24-alpine

WORKDIR /app
COPY --from=build /app/build ./build
COPY --from=build /app/node_modules ./node_modules
COPY server.js ./

EXPOSE 3000
CMD ["sh", "-lc", "HOST=0.0.0.0 PORT=${DASHBOARD_PORT:-3000} node server.js"]
