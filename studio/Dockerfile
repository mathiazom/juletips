FROM node:25-alpine

ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
ENV CI="true"
ENV PNPM_VERSION=10.26.0

ENV SANITY_STUDIO_DATASET="production"
ENV SANITY_STUDIO_PROJECT_ID="1xz4z3wg"
ENV SANITY_STUDIO_BASEPATH="/admin"

RUN wget -qO- https://get.pnpm.io/install.sh | ENV="$HOME/.shrc" SHELL="$(which sh)" sh -
RUN apk add --no-cache apache2

WORKDIR /app
COPY . .

RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile

RUN pnpm run build

EXPOSE 80

RUN npx serve --version

CMD ["npx", "serve", "-s", "dist", "-l", "80"]
