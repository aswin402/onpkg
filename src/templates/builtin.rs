use crate::templates::{TemplateDefinition, TemplateFile, TemplateVariable};

pub fn builtin_templates() -> Vec<TemplateDefinition> {
    vec![
        // ── React + Vite (Frontend) ────────────────────────────────
        react_vite(),
        react_vite_tailwind(),
        // ── Next.js (Frontend/Fullstack) ────────────────────────────
        next_app(),
        next_app_full(),
        // ── Hono API (Backend) ─────────────────────────────────────
        hono_api(),
        hono_api_full(),
        // ── FastAPI (Backend) ──────────────────────────────────────
        fastapi(),
        fastapi_full(),
        // ── Express API (Backend) ──────────────────────────────────
        express_api(),
        // ── MERN (Fullstack) ───────────────────────────────────────
        mern(),
        pern(),
        // ── Flutter App ────────────────────────────────────────────
        flutter_app(),
        // ── Static Website ─────────────────────────────────────────
        static_website(),
        // ── Minimal (General) ──────────────────────────────────────
        minimal_rust_cli(),
    ]
}

// ── React + Vite ──────────────────────────────────────────────────────────

fn react_vite() -> TemplateDefinition {
    TemplateDefinition {
        name: "react-vite".to_string(),
        category: "frontend".to_string(),
        description: "React 19 + Vite + TypeScript starter with ESLint".to_string(),
        version: "1.0.0".to_string(),
        variables: vec![
            TemplateVariable {
                name: "project_name".to_string(),
                description: "Project name".to_string(),
                default: "my-app".to_string(),
            },
            TemplateVariable {
                name: "author".to_string(),
                description: "Author name".to_string(),
                default: "developer".to_string(),
            },
        ],
        technologies: vec![],
        files: vec![
            TemplateFile {
                path: "package.json".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"{
  "name": "{{ project_name }}",
  "private": true,
  "version": "0.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "react": "^19.0.0",
    "react-dom": "^19.0.0"
  },
  "devDependencies": {
    "@types/react": "^19.0.0",
    "@types/react-dom": "^19.0.0",
    "@vitejs/plugin-react": "^4.3.0",
    "typescript": "^5.6.0",
    "vite": "^6.0.0"
  }
}"#
                .to_string(),
            },
            TemplateFile {
                path: "tsconfig.json".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "isolatedModules": true,
    "moduleDetection": "force",
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "noUncheckedSideEffectImports": true
  },
  "include": ["src"]
}"#
                .to_string(),
            },
            TemplateFile {
                path: "vite.config.ts".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
})"#
                .to_string(),
            },
            TemplateFile {
                path: "index.html".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{{ project_name }}</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>"#
                    .to_string(),
            },
            TemplateFile {
                path: "src/main.tsx".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import App from './App.tsx'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)"#
                .to_string(),
            },
            TemplateFile {
                path: "src/App.tsx".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"function App() {
  return (
    <div>
      <h1>Hello {{ project_name }}</h1>
    </div>
  )
}

export default App"#
                    .to_string(),
            },
            TemplateFile {
                path: "src/vite-env.d.ts".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"/// <reference types="vite/client" />
"#
                .to_string(),
            },
        ],
    }
}

fn react_vite_tailwind() -> TemplateDefinition {
    TemplateDefinition {
        name: "react-vite-tailwind".to_string(),
        category: "frontend".to_string(),
        description: "React 19 + Vite + TypeScript + Tailwind CSS v4".to_string(),
        version: "1.0.0".to_string(),
        variables: vec![TemplateVariable {
            name: "project_name".to_string(),
            description: "Project name".to_string(),
            default: "my-app".to_string(),
        }],
        technologies: vec![],
        files: vec![
            TemplateFile {
                path: "package.json".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"{
  "name": "{{ project_name }}",
  "private": true,
  "version": "0.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "react": "^19.0.0",
    "react-dom": "^19.0.0"
  },
  "devDependencies": {
    "@tailwindcss/vite": "^4.0.0",
    "@types/react": "^19.0.0",
    "@types/react-dom": "^19.0.0",
    "@vitejs/plugin-react": "^4.3.0",
    "tailwindcss": "^4.0.0",
    "typescript": "^5.6.0",
    "vite": "^6.0.0"
  }
}"#
                .to_string(),
            },
            TemplateFile {
                path: "tsconfig.json".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "isolatedModules": true,
    "moduleDetection": "force",
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "noUncheckedSideEffectImports": true
  },
  "include": ["src"]
}"#
                .to_string(),
            },
            TemplateFile {
                path: "vite.config.ts".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [react(), tailwindcss()],
})"#
                .to_string(),
            },
            TemplateFile {
                path: "index.html".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{{ project_name }}</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>"#
                    .to_string(),
            },
            TemplateFile {
                path: "src/main.tsx".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import App from './App.tsx'
import './index.css'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)"#
                .to_string(),
            },
            TemplateFile {
                path: "src/App.tsx".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"function App() {
  return (
    <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
      <h1 className="text-4xl font-bold">Hello {{ project_name }}</h1>
    </div>
  )
}

export default App"#
                    .to_string(),
            },
            TemplateFile {
                path: "src/index.css".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"@import "tailwindcss";
"#
                .to_string(),
            },
            TemplateFile {
                path: "src/vite-env.d.ts".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"/// <reference types="vite/client" />
"#
                .to_string(),
            },
        ],
    }
}

// ── Next.js ───────────────────────────────────────────────────────────────

fn next_app() -> TemplateDefinition {
    TemplateDefinition {
        name: "next-app".to_string(),
        category: "frontend".to_string(),
        description: "Next.js 15 App Router + TypeScript starter".to_string(),
        version: "1.0.0".to_string(),
        variables: vec![TemplateVariable {
            name: "project_name".to_string(),
            description: "Project name".to_string(),
            default: "my-app".to_string(),
        }],
        technologies: vec![],
        files: vec![
            TemplateFile {
                path: "package.json".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"{
  "name": "{{ project_name }}",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "dev": "next dev",
    "build": "next build",
    "start": "next start",
    "lint": "next lint"
  },
  "dependencies": {
    "next": "^15.0.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0"
  },
  "devDependencies": {
    "@types/node": "^22.0.0",
    "@types/react": "^19.0.0",
    "@types/react-dom": "^19.0.0",
    "typescript": "^5.6.0"
  }
}"#
                .to_string(),
            },
            TemplateFile {
                path: "tsconfig.json".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"{
  "compilerOptions": {
    "target": "ES2017",
    "lib": ["dom", "dom.iterable", "esnext"],
    "allowJs": true,
    "skipLibCheck": true,
    "strict": true,
    "noEmit": true,
    "esModuleInterop": true,
    "module": "esnext",
    "moduleResolution": "bundler",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "jsx": "preserve",
    "incremental": true,
    "plugins": [{ "name": "next" }],
    "paths": { "@/*": ["./src/*"] }
  },
  "include": ["next-env.d.ts", "**/*.ts", "**/*.tsx", ".next/types/**/*.ts"],
  "exclude": ["node_modules"]
}"#
                .to_string(),
            },
            TemplateFile {
                path: "next.config.ts".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"import type { NextConfig } from "next";

const nextConfig: NextConfig = {};

export default nextConfig;"#
                    .to_string(),
            },
            TemplateFile {
                path: "src/app/layout.tsx".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "{{ project_name }}",
  description: "Generated by onpkg",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}"#
                .to_string(),
            },
            TemplateFile {
                path: "src/app/page.tsx".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"export default function Home() {
  return <h1>Hello {{ project_name }}</h1>;
}"#
                .to_string(),
            },
            TemplateFile {
                path: "src/app/globals.css".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"body { font-family: system-ui, sans-serif; }"#.to_string(),
            },
        ],
    }
}

fn next_app_full() -> TemplateDefinition {
    TemplateDefinition {
        name: "next-app-full".to_string(),
        category: "frontend".to_string(),
        description: "Next.js 15 + Tailwind CSS + shadcn/ui + Prisma + Auth.js".to_string(),
        version: "1.0.0".to_string(),
        variables: vec![TemplateVariable {
            name: "project_name".to_string(),
            description: "Project name".to_string(),
            default: "my-app".to_string(),
        }],
        technologies: vec![],
        files: vec![
            TemplateFile {
                path: "package.json".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"{
  "name": "{{ project_name }}",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "dev": "next dev",
    "build": "next build",
    "start": "next start",
    "lint": "next lint"
  },
  "dependencies": {
    "next": "^15.0.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "@prisma/client": "^6.0.0",
    "next-auth": "^5.0.0",
    "zod": "^3.23.0"
  },
  "devDependencies": {
    "@types/node": "^22.0.0",
    "@types/react": "^19.0.0",
    "@types/react-dom": "^19.0.0",
    "prisma": "^6.0.0",
    "tailwindcss": "^4.0.0",
    "typescript": "^5.6.0"
  }
}"#
                .to_string(),
            },
            TemplateFile {
                path: "src/app/layout.tsx".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "{{ project_name }}",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}"#
                .to_string(),
            },
            TemplateFile {
                path: "src/app/globals.css".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"@import "tailwindcss";
"#
                .to_string(),
            },
        ],
    }
}

// ── Hono API ──────────────────────────────────────────────────────────────

fn hono_api() -> TemplateDefinition {
    TemplateDefinition {
        name: "hono-api".to_string(),
        category: "backend".to_string(),
        description: "Hono + TypeScript API server with Zod validation".to_string(),
        version: "1.0.0".to_string(),
        variables: vec![TemplateVariable {
            name: "project_name".to_string(),
            description: "Project name".to_string(),
            default: "my-api".to_string(),
        }],
        technologies: vec![],
        files: vec![
            TemplateFile {
                path: "package.json".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"{
  "name": "{{ project_name }}",
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "tsx watch src/index.ts",
    "build": "tsc",
    "start": "node dist/index.js"
  },
  "dependencies": {
    "hono": "^4.6.0",
    "zod": "^3.23.0"
  },
  "devDependencies": {
    "@types/node": "^22.0.0",
    "tsx": "^4.19.0",
    "typescript": "^5.6.0"
  }
}"#
                .to_string(),
            },
            TemplateFile {
                path: "tsconfig.json".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "outDir": "dist",
    "rootDir": "src",
    "strict": true,
    "skipLibCheck": true,
    "esModuleInterop": true,
    "declaration": true
  },
  "include": ["src"]
}"#
                .to_string(),
            },
            TemplateFile {
                path: "src/index.ts".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"import { Hono } from 'hono'

const app = new Hono()

app.get('/', (c) => c.json({ message: 'Hello from {{ project_name }}' }))

export default app"#
                    .to_string(),
            },
        ],
    }
}

fn hono_api_full() -> TemplateDefinition {
    TemplateDefinition {
        name: "hono-full".to_string(),
        category: "backend".to_string(),
        description: "Hono + Prisma + PostgreSQL + Auth + Zod".to_string(),
        version: "1.0.0".to_string(),
        variables: vec![TemplateVariable {
            name: "project_name".to_string(),
            description: "Project name".to_string(),
            default: "my-api".to_string(),
        }],
        technologies: vec![],
        files: vec![
            TemplateFile {
                path: "package.json".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"{
  "name": "{{ project_name }}",
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "tsx watch src/index.ts",
    "build": "tsc",
    "db:push": "prisma db push",
    "db:generate": "prisma generate"
  },
  "dependencies": {
    "hono": "^4.6.0",
    "@prisma/client": "^6.0.0",
    "zod": "^3.23.0",
    "jsonwebtoken": "^9.0.0"
  },
  "devDependencies": {
    "@types/node": "^22.0.0",
    "@types/jsonwebtoken": "^9.0.0",
    "prisma": "^6.0.0",
    "tsx": "^4.19.0",
    "typescript": "^5.6.0"
  }
}"#
                .to_string(),
            },
            TemplateFile {
                path: "src/index.ts".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"import { Hono } from 'hono'

const app = new Hono()

app.get('/', (c) => c.json({ message: 'Hello from {{ project_name }}' }))

export default app"#
                    .to_string(),
            },
            TemplateFile {
                path: "prisma/schema.prisma".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"generator client {
  provider = "prisma-client-js"
}

datasource db {
  provider = "postgresql"
  url      = env("DATABASE_URL")
}

model User {
  id        String   @id @default(cuid())
  email     String   @unique
  name      String?
  createdAt DateTime @default(now())
  updatedAt DateTime @updatedAt
}"#
                .to_string(),
            },
            TemplateFile {
                path: "".to_string() + ".env",
                skip_template: false,
                binary_content: None,
                content: r#"DATABASE_URL="postgresql://localhost:5432/{{ project_name }}"
JWT_SECRET="change-me"
"#
                .to_string(),
            },
        ],
    }
}

// ── Express API ───────────────────────────────────────────────────────────

fn express_api() -> TemplateDefinition {
    TemplateDefinition {
        name: "express-api".to_string(),
        category: "backend".to_string(),
        description: "Express.js + TypeScript REST API with middleware".to_string(),
        version: "1.0.0".to_string(),
        variables: vec![TemplateVariable {
            name: "project_name".to_string(),
            description: "Project name".to_string(),
            default: "my-api".to_string(),
        }],
        technologies: vec![],
        files: vec![
            TemplateFile {
                path: "package.json".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"{
  "name": "{{ project_name }}",
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "tsx watch src/index.ts",
    "build": "tsc",
    "start": "node dist/index.js"
  },
  "dependencies": {
    "express": "^5.0.0",
    "cors": "^2.8.5",
    "helmet": "^8.0.0",
    "morgan": "^1.10.0"
  },
  "devDependencies": {
    "@types/express": "^5.0.0",
    "@types/node": "^22.0.0",
    "tsx": "^4.19.0",
    "typescript": "^5.6.0"
  }
}"#
                .to_string(),
            },
            TemplateFile {
                path: "src/index.ts".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"import express from "express"
import cors from "cors"
import helmet from 'helmet'
import morgan from 'morgan'

const app = express()
const port = process.env.PORT || 3000

app.use(helmet())
app.use(cors())
app.use(morgan('dev'))
app.use(express.json())

app.get('/', (_req, res) => {
  res.json({ message: 'Hello from {{ project_name }}' })
})

app.listen(port, () => {
  console.log(`Server running on port ${port}`)
})"#
                .to_string(),
            },
        ],
    }
}

// ── FastAPI ───────────────────────────────────────────────────────────────

fn fastapi() -> TemplateDefinition {
    TemplateDefinition {
        name: "fastapi".to_string(),
        category: "backend".to_string(),
        description: "FastAPI + SQLAlchemy + Pydantic + Alembic starter".to_string(),
        version: "1.0.0".to_string(),
        variables: vec![TemplateVariable {
            name: "project_name".to_string(),
            description: "Project name".to_string(),
            default: "my-api".to_string(),
        }],
        technologies: vec![],
        files: vec![
            TemplateFile {
                path: "pyproject.toml".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"[project]
name = "{{ project_name }}"
version = "0.1.0"
requires-python = ">=3.12"
dependencies = [
    "fastapi>=0.115.0",
    "uvicorn[standard]>=0.32.0",
    "sqlalchemy>=2.0.0",
    "pydantic-settings>=2.0.0",
    "alembic>=1.13.0",
    "psycopg2-binary>=2.9.0",
]
[dependency-groups]
dev = ["pytest>=8.0.0"]
"#
                .to_string(),
            },
            TemplateFile {
                path: "src/main.py".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"from fastapi import FastAPI

app = FastAPI(title="{{ project_name }}")

@app.get("/")
async def root():
    return {"message": "Hello from {{ project_name }}"}
"#
                .to_string(),
            },
            TemplateFile {
                path: "src/config.py".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"from pydantic_settings import BaseSettings

class Settings(BaseSettings):
    database_url: str = "postgresql://localhost:5432/{{ project_name }}"
    debug: bool = False

    class Config:
        env_file = ".env"

settings = Settings()
"#
                .to_string(),
            },
            TemplateFile {
                path: "".to_string() + ".env",
                skip_template: false,
                binary_content: None,
                content: r#"DATABASE_URL="postgresql://localhost:5432/{{ project_name }}"
DEBUG=true
"#
                .to_string(),
            },
        ],
    }
}

fn fastapi_full() -> TemplateDefinition {
    TemplateDefinition {
        name: ("fastapi-full").to_string(),
        category: "backend".to_string(),
        description: ("FastAPI + SQLAlchemy + Auth + Celery + Docker").to_string(),
        version: "1.0.0".to_string(),
        variables: vec![TemplateVariable {
            name: "project_name".to_string(),
            description: "Project name".to_string(),
            default: "my-api".to_string(),
        }],
        technologies: vec![],
        files: vec![
            TemplateFile {
                path: "pyproject.toml".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"[project]
name = "{{ project_name }}"
version = "0.1.0"
requires-python = ">=3.12"
dependencies = [
    "fastapi>=0.115.0",
    "uvicorn[standard]>=0.32.0",
    "sqlalchemy>=2.0.0",
    "pydantic-settings>=2.0.0",
    "alembic>=1.13.0",
    "psycopg2-binary>=2.9.0",
    "python-jose[cryptography]>=3.3.0",
    "passlib[bcrypt]>=1.7.0",
    "celery>=5.4.0",
    "redis>=5.0.0",
]
[dependency-groups]
dev = ["pytest>=8.0.0", "httpx>=0.27.0"]
"#
                .to_string(),
            },
            TemplateFile {
                path: "src/main.py".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"from fastapi import FastAPI

app = FastAPI(title="{{ project_name }}")

@app.get("/")
async def root():
    return {"message": "Hello from {{ project_name }}"}
"#
                .to_string(),
            },
            TemplateFile {
                path: "Dockerfile".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"FROM python:3.12-slim
WORKDIR /app
COPY . .
RUN pip install .
CMD ["uvicorn", "src.main:app", "--host", "0.0.0.0", "--port", "8000"]
"#
                .to_string(),
            },
            TemplateFile {
                path: "docker-compose.yml".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"version: "3.9"
services:
  api:
    build: .
    ports:
      - "8000:8000"
    env_file: .env
  db:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: {{ project_name }}
      POSTGRES_PASSWORD: password
    ports:
      - "5432:5432"
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
"#
                .to_string(),
            },
        ],
    }
}

// ── MERN ──────────────────────────────────────────────────────────────────

fn mern() -> TemplateDefinition {
    TemplateDefinition {
        name: "mern".to_string(),
        category: "fullstack".to_string(),
        description: ("MongoDB + Express + React + Node.js fullstack app").to_string(),
        version: "1.0.0".to_string(),
        variables: vec![TemplateVariable {
            name: "project_name".to_string(),
            description: "Project name".to_string(),
            default: "my-app".to_string(),
        }],
        technologies: vec![],
        files: vec![
            TemplateFile {
                path: "package.json".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"{
  "name": "{{ project_name }}",
  "private": true,
  "scripts": {
    "dev": "concurrently \"npm run dev:server\" \"npm run dev:client\"",
    "dev:server": "cd server && npm run dev",
    "dev:client": "cd client && npm run dev"
  },
  "devDependencies": {
    "concurrently": "^9.0.0"
  }
}"#
                .to_string(),
            },
            TemplateFile {
                path: "server/package.json".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"{
  "name": "{{ project_name }}-server",
  "type": "module",
  "scripts": {
    "dev": "tsx watch src/index.ts"
  },
  "dependencies": {
    "express": "^5.0.0",
    "mongoose": "^8.0.0",
    "cors": "^2.8.5",
    "dotenv": "^16.4.0"
  },
  "devDependencies": {
    "@types/express": "^5.0.0",
    "@types/node": "^22.0.0",
    "tsx": "^4.19.0",
    "typescript": "^5.6.0"
  }
}"#
                .to_string(),
            },
            TemplateFile {
                path: "server/src/index.ts".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"import express from "express"
import mongoose from "mongoose"
import cors from "cors"
import dotenv from "dotenv"

dotenv.config()

const app = express()
app.use(cors())
app.use(express.json())

mongoose.connect(process.env.MONGODB_URI || 'mongodb://localhost:27017/{{ project_name }}')
  .then(() => console.log("Connected to MongoDB"))

app.get('/', (_req, res) => res.json({ message: 'Hello from {{ project_name }}' }))

app.listen(process.env.PORT || 5000, () => console.log("Server running"))"#
                    .to_string(),
            },
            TemplateFile {
                path: "client/package.json".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"{
  "name": "{{ project_name }}-client",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build"
  },
  "dependencies": {
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "react-router-dom": "^7.0.0"
  },
  "devDependencies": {
    "@types/react": "^19.0.0",
    "@types/react-dom": "^19.0.0",
    "@vitejs/plugin-react": "^4.3.0",
    "typescript": "^5.6.0",
    "vite": "^6.0.0"
  }
}"#
                .to_string(),
            },
        ],
    }
}

fn pern() -> TemplateDefinition {
    TemplateDefinition {
        name: "pern".to_string(),
        category: "fullstack".to_string(),
        description: "PostgreSQL + Express + React + Node.js fullstack app".to_string(),
        version: "1.0.0".to_string(),
        variables: vec![TemplateVariable {
            name: "project_name".to_string(),
            description: "Project name".to_string(),
            default: "my-app".to_string(),
        }],
        technologies: vec![],
        files: vec![
            TemplateFile {
                path: "package.json".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"{
  "name": "{{ project_name }}",
  "private": true,
  "scripts": {
    "dev": "concurrently \"npm run dev:server\" \"npm run dev:client\"",
    "dev:server": "cd server && npm run dev",
    "dev:client": "cd client && npm run dev"
  },
  "devDependencies": { "concurrently": "^9.0.0" }
}"#
                .to_string(),
            },
            TemplateFile {
                path: "server/package.json".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"{
  "name": "{{ project_name }}-server",
  "type": "module",
  "scripts": { "dev": "tsx watch src/index.ts" },
  "dependencies": {
    "express": "^5.0.0",
    "@prisma/client": "^6.0.0",
    "cors": "^2.8.5",
    "zod": "^3.23.0"
  },
  "devDependencies": {
    "@types/express": "^5.0.0",
    "prisma": "^6.0.0",
    "tsx": "^4.19.0",
    "typescript": "^5.6.0"
  }
}"#
                .to_string(),
            },
        ],
    }
}

// ── Flutter ───────────────────────────────────────────────────────────────

fn flutter_app() -> TemplateDefinition {
    TemplateDefinition {
        name: "flutter-app".to_string(),
        category: "app".to_string(),
        description: "Flutter + Riverpod + GoRouter starter app".to_string(),
        version: "1.0.0".to_string(),
        variables: vec![TemplateVariable {
            name: "project_name".to_string(),
            description: "Project name".to_string(),
            default: "my_app".to_string(),
        }],
        technologies: vec![],
        files: vec![
            TemplateFile {
                path: "pubspec.yaml".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"name: {{ project_name }}
description: "Generated by onpkg"
publish_to: none
version: 1.0.0+1

environment:
  sdk: ^3.5.0

dependencies:
  flutter:
    sdk: flutter
  flutter_riverpod: ^2.6.0
  go_router: ^14.0.0

dev_dependencies:
  flutter_test:
    sdk: flutter
  flutter_lints: ^5.0.0

flutter:
  uses-material-design: true
"#
                .to_string(),
            },
            TemplateFile {
                path: "lib/main.dart".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

void main() {
  runApp(const ProviderScope(child: MyApp()));
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: '{{ project_name }}',
      theme: ThemeData(
        colorSchemeSeed: Colors.blue,
        useMaterial3: true,
      ),
      home: const HomePage(),
    );
  }
}

class HomePage extends StatelessWidget {
  const HomePage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('{{ project_name }}')),
      body: const Center(child: Text('Hello!')),
    );
  }
}
"#
                .to_string(),
            },
        ],
    }
}

// ── Static Website ────────────────────────────────────────────────────────

fn static_website() -> TemplateDefinition {
    TemplateDefinition {
        name: "static-website".to_string(),
        category: "website".to_string(),
        description: "Minimal HTML5 + CSS3 + JS static website".to_string(),
        version: "1.0.0".to_string(),
        variables: vec![
            TemplateVariable { name: "project_name".to_string(), description: "Project name".to_string(), default: "my-site".to_string() },
            TemplateVariable { name: "author".to_string(), description: "Author name".to_string(), default: "Developer".to_string() },
        ],
        technologies: vec![],
        files: vec![
            TemplateFile { path: "index.html".to_string(), skip_template: false, binary_content: None, content: r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>{{ project_name }}</title>
  <link rel="stylesheet" href="style.css" />
</head>
<body>
  <h1>Welcome to {{ project_name }}</h1>
  <p>Built by {{ author }}</p>
  <script src="app.js"></script>
</body>
</html>"#.to_string() },
            TemplateFile { path: "style.css".to_string(), skip_template: false, binary_content: None, content: r#"* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: system-ui, sans-serif; display: grid; place-items: center; min-height: 100vh; background: #0f172a; color: #e2e8f0; }
h1 { font-size: 2.5rem; }
"#.to_string() },
            TemplateFile { path: "app.js".to_string(), skip_template: false, binary_content: None, content: r#"console.log("Hello from {{ project_name }}");
"#.to_string() },
        ],
    }
}

// ── Minimal Rust CLI ──────────────────────────────────────────────────────

fn minimal_rust_cli() -> TemplateDefinition {
    TemplateDefinition {
        name: "rust-cli".to_string(),
        category: "app".to_string(),
        description: "Minimal Rust CLI app with clap and anyhow".to_string(),
        version: "1.0.0".to_string(),
        variables: vec![TemplateVariable {
            name: "project_name".to_string(),
            description: "Project name".to_string(),
            default: "my-cli".to_string(),
        }],
        technologies: vec![],
        files: vec![
            TemplateFile {
                path: "Cargo.toml".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"[package]
name = "{{ project_name }}"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4.5", features = ["derive"] }
anyhow = "1.0"
"#
                .to_string(),
            },
            TemplateFile {
                path: ".cargo/config.toml".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"[build]
# Limit parallel compilation jobs to conserve CPU and RAM
jobs = 2

[target.'cfg(target_os = "linux")']
# Limit the number of threads used by the linker to reduce peak RAM usage on Linux
rustflags = ["-C", "link-arg=-Wl,--threads=2"]
"#
                .to_string(),
            },
            TemplateFile {
                path: "src/main.rs".to_string(),
                skip_template: false,
                binary_content: None,
                content: r#"use clap::Parser;
 
 #[derive(Parser)]
 #[command(name = "{{ project_name }}", version = "0.1.0")]
 struct Args {
     name: Option<String>,
 }
 
 fn main() -> anyhow::Result<()> {
     let args = Args::parse();
     let name = args.name.unwrap_or_else(|| "World".to_string());
     println!("Hello, {}!", name);
     Ok(())
 }
 "#
                .to_string(),
            },
        ],
    }
}
