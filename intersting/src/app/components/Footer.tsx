import React from 'react';
import { Anchor, Heart, Shield } from 'lucide-react';

export function Footer() {
  return (
    <footer className="border-t border-slate-900 bg-[#070a12] px-6 lg:px-12 py-12">
      <div className="max-w-7xl mx-auto flex flex-col sm:flex-row items-center justify-between gap-6">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-cyan-500/10 border border-cyan-500/30 flex items-center justify-center text-cyan-400">
            <Anchor className="w-4 h-4" />
          </div>
          <div>
            <span className="font-['Space_Grotesk'] text-sm font-bold tracking-tight text-white block">
              DOCKWORKER.AI
            </span>
            <span className="text-[11px] text-slate-500 font-mono">
              A Lornu AI Research Standard
            </span>
          </div>
        </div>

        <div className="flex items-center gap-6 text-xs text-slate-400 font-mono">
          <a href="https://www.aivcs.io" className="hover:text-cyan-400 transition-colors">aivcs.io</a>
          <a href="https://lornu.ai" className="hover:text-cyan-400 transition-colors">Lornu AI</a>
          <a href="/privacy" className="hover:text-cyan-400 transition-colors">Privacy Policy</a>
          <a href="/terms" className="hover:text-cyan-400 transition-colors">Terms of Service</a>
          <a href="https://stevedores.org" className="hover:text-cyan-400 transition-colors">Stevedores</a>
        </div>

        <div className="text-xs text-slate-500 font-mono flex items-center gap-1.5">
          <Shield className="w-3.5 h-3.5 text-emerald-400" />
          <span>Option 3 Canonical Spec</span>
        </div>
      </div>
    </footer>
  );
}
