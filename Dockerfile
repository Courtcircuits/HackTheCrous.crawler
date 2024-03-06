# this is a job image
FROM node:21-alpine3.18 AS build
WORKDIR /app
COPY . .
RUN npm install && npm run build

FROM node:21-alpine3.18 AS production
WORKDIR /app
COPY --from=build /app/dist /app/dist
COPY --from=build /app/node_modules /app/node_modules
COPY --from=build /app/script /app/script
CMD /bin/bash -c