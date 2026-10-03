import React from 'react';
import { ShieldCheck, Cpu, Box, CheckCircle2, ArrowRight, Zap, Terminal } from 'lucide-react';

export function HeroSection() {
  return (
    <section className="relative pt-24 pb-20 px-6 lg:px-12 overflow-hidden">
      {/* Glow effects */}
      <div className="absolute top-1/4 left-1/2 -translate-x-1/2 w-[700px] h-[350px] bg-gradient-to-tr from-cyan-500/15 via-blue-600/10 to-transparent blur-3xl pointer-events-none -z-10" />

      <div className="max-w-7xl mx-auto grid lg:grid-cols-[1.1fr_0.9fr] gap-12 items-center">
        {/* Left column */}
        <div className="space-y-8">
          <div className="inline-flex items-center gap-2.5 px-3.5 py-1.5 rounded-full bg-cyan-950/50 border border-cyan-500/30 text-xs font-mono text-cyan-300">
            <span className="w-2 h-2 rounded-full bg-cyan-400 animate-pulse" />
            <span>Standard: Option 3 Canonical</span>
          </div>

          <h1 className="font-['Space_Grotesk'] text-4xl sm:text-5xl lg:text-6xl font-extrabold tracking-tight text-white leading-[1.1]">
            Deterministic Container Packaging &{' '}
            <span className="bg-gradient-to-r from-cyan-400 via-teal-300 to-blue-500 bg-clip-text text-transparent">
              OCI Build Engine
            </span>
          </h1>

          <p className="text-base sm:text-lg text-slate-300 leading-relaxed max-w-2xl">
            Compile hermetic, reproducible container environments and microservices natively with Nix Flakes & APKO. Zero Dockerfiles, zero drift, and Merkle-tree provenance.
          </p>

          <div className="flex flex-wrap items-center gap-4 pt-2">
            <a
              href="#spec"
              className="inline-flex items-center gap-2 px-6 py-3 rounded-xl bg-cyan-400 text-slate-950 font-bold text-sm hover:bg-cyan-300 transition-all shadow-lg shadow-cyan-500/25"
            >
              <span>Explore dockworker.toml</span>
              <ArrowRight className="w-4 h-4" />
            </a>
            <a
              href="#matrix"
              className="inline-flex items-center gap-2 px-6 py-3 rounded-xl bg-slate-900 border border-slate-700/80 text-white font-semibold text-sm hover:border-slate-500 transition-colors"
            >
              <Terminal className="w-4 h-4 text-cyan-400" />
              <span>Propel CI Webhooks</span>
            </a>
          </div>

          {/* Badges */}
          <div className="grid grid-cols-2 sm:grid-cols-3 gap-3 pt-4 border-t border-slate-800/80">
            <div className="flex items-center gap-2 text-xs font-mono text-slate-300">
              <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
              <span>Pure Nix Sandbox</span>
            </div>
            <div className="flex items-center gap-2 text-xs font-mono text-slate-300">
              <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
              <span>Never Dockerfile</span>
            </div>
            <div className="flex items-center gap-2 text-xs font-mono text-slate-300">
              <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
              <span>GCP GAR & AWS ECR</span>
            </div>
          </div>
        </div>

        {/* Right column: Terminal preview */}
        <div className="rounded-2xl border border-slate-800 bg-[#0d1322]/90 shadow-2xl overflow-hidden backdrop-blur-xl">
          <div className="flex items-center justify-between px-4 py-3 bg-slate-900/90 border-b border-slate-800 text-xs font-mono text-slate-400">
            <div className="flex items-center gap-2">
              <div className="w-3 h-3 rounded-full bg-red-500/80" />
              <div className="w-3 h-3 rounded-full bg-yellow-500/80" />
              <div className="w-3 h-3 rounded-full bg-green-500/80" />
              <span className="ml-2 text-slate-300 font-semibold">dockworker.toml</span>
            </div>
            <span className="text-[11px] text-cyan-400">OCI Engine v2.0</span>
          </div>
          <div className="p-6 font-mono text-xs text-slate-300 space-y-2 leading-relaxed overflow-x-auto">
            <div className="text-slate-500"># Universal 1:1:1 Declarative Packaging</div>
            <div><span className="text-cyan-400">[build]</span></div>
            <div>name = <span className="text-emerald-400">"agent-envelope-ai"</span></div>
            <div>version = <span className="text-emerald-400">"0.2.0"</span></div>
            <div>engine = <span className="text-emerald-400">"nix"</span></div>
            <div>target = <span className="text-emerald-400">"packages.x86_64-linux.oci"</span></div>
            <br />
            <div><span className="text-cyan-400">[distribution]</span></div>
            <div>registries = [</div>
            <div className="pl-4 text-amber-300">"us-docker.pkg.dev/gcp-lornu-ai/aivcs-oci",</div>
            <div className="pl-4 text-amber-300">"177559565127.dkr.ecr.us-east-2.amazonaws.com/aivcs"</div>
            <div>]</div>
            <br />
            <div><span className="text-cyan-400">[deploy]</span></div>
            <div>namespace = <span className="text-emerald-400">"agent-envelope"</span></div>
            <div>manifest = <span className="text-emerald-400">"apps/agent-envelope/base"</span></div>
            <div className="text-slate-500 pt-2"># Verified via Fast Free Testing (FFT) Gate</div>
            <div className="text-cyan-300 flex items-center gap-1.5 pt-1">
              <Zap className="w-3.5 h-3.5 text-amber-400" />
              <span>✓ Compiled & verified in pure sandbox (0 warnings)</span>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
