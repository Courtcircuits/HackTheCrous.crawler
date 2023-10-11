declare global {
    namespace NodeJS {
        interface ProcessEnv {
            DATABASE_URL: string;
            PG_DATABASE: string;
            PG_HOST: string;
            PG_PASSWORD: string;
            PG_PORT: string;
            PG_USER:string;
        }
    }
}

export {}