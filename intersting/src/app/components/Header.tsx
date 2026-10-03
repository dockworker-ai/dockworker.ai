import React from 'react';
import { Anchor, ShieldCheck, Terminal, BookOpen, GitBranch } from 'lucide-react';

export function Header() {
  return (
    <header className="sticky top-0 z-50 backdrop-blur-xl bg-[#0b0f19]/80 border-b border-slate-800/80 px-6 lg:px-12 py-4">
      <div className="max-w-7xl mx-auto flex items-center justify-between">
        <a href="/" className="flex items-center gap-3">
          <div className="w-9 h-9 rounded-lg bg-gradient-to-br from-cyan-400 to-blue-600 flex items-center justify-center text-slate-950 font-bold shadow-lg shadow-cyan-500/20">
            <Anchor className="w-5 h-5 text-black stroke-[2.5]" />
          </div>
          <div>
            <span className="font-['Space_Grotesk'] text-lg font-bold tracking-tight text-white flex items-center gap-1.5">
              DOCKWORKER<span className="text-cyan-400">.AI</span>
            </span>
            <span className="text-[10px] font-mono text-slate-400 uppercase tracking-widest block -mt-1">
              Deterministic OCI Engine
            </span>
          </div>
        </a>

        <nav className="hidden md:flex items-center gap-8 text-sm font-medium text-slate-400">
          <a href="#spec" className="hover:text-cyan-400 transition-colors">Specification</a>
          <a href="#matrix" className="hover:text-cyan-400 transition-colors">Build Matrix</a>
          <a href="#zero-dockerfile" className="hover:text-cyan-400 transition-colors">Zero-Dockerfile Policy</a>
          <a href="https://www.aivcs.io/docs" className="hover:text-cyan-400 transition-colors flex items-center gap-1">
            <BookOpen className="w-3.5 h-3.5" /> Docs
          </a>
        </nav>

        <div className="flex items-center gap-3">
          <a
            href="https://www.aivcs.io"
            className="hidden sm:inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-slate-900 border border-slate-800 text-xs font-mono text-slate-300 hover:border-cyan-500/50 hover:text-white transition-all shadow-sm"
          >
            <Terminal className="w-3.5 h-3.5 text-cyan-400" />
            <span>aivcs push</span>
          </a>
          <a
            href="https://www.aivcs.io"
            className="inline-flex items-center gap-1.5 px-4 py-2 rounded-lg bg-gradient-to-r from-cyan-500 to-blue-600 text-slate-950 font-semibold text-xs hover:opacity-90 transition-opacity shadow-md shadow-cyan-500/20"
          >
            <GitBranch className="w-3.5 h-3.5" />
            <span>Forge</span>
          </a>
        </div>
      </div>
    </header>
  );
}
