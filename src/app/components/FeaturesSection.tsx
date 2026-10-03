import React from 'react';
import { Cpu, ShieldCheck, Box, RefreshCw, GitBranch, Layers, Lock, Sparkles } from 'lucide-react';

export function FeaturesSection() {
  const features = [
    {
      icon: Cpu,
      title: "Pure Nix Derivations",
      desc: "Hermetic sandbox compilation guarantees bit-for-bit reproducible container environments and microservices without non-deterministic dependencies.",
      badge: "Reproducible",
      color: "from-cyan-500/20 to-blue-600/10",
      border: "border-cyan-500/30",
      iconColor: "text-cyan-400"
    },
    {
      icon: ShieldCheck,
      title: "Never Dockerfile Standard",
      desc: "Zero mutable runtime state. All container layers are declared mathematically via Nix Flakes expressions and compiled securely without root daemons.",
      badge: "Zero-Trust",
      color: "from-emerald-500/20 to-teal-600/10",
      border: "border-emerald-500/30",
      iconColor: "text-emerald-400"
    },
    {
      icon: Box,
      title: "Universal 1:1:1 Fleet Mapping",
      desc: "1 Git Repository = 1 Propel Build Entry (`dockworker.toml`) = 1 Declarative Kubernetes / Cloudflare Pages manifest (`deploy/apps/`).",
      badge: "Architecture",
      color: "from-purple-500/20 to-indigo-600/10",
      border: "border-purple-500/30",
      iconColor: "text-purple-400"
    },
    {
      icon: RefreshCw,
      title: "OxidizedFlux Reconciler Loop",
      desc: "Declarative GitOps synchronization continuously compares live cluster pods against forge tags, self-healing configuration drift.",
      badge: "GitOps",
      color: "from-amber-500/20 to-orange-600/10",
      border: "border-amber-500/30",
      iconColor: "text-amber-400"
    }
  ];

  return (
    <section id="matrix" className="py-24 px-6 lg:px-12 bg-slate-950/40 border-t border-slate-900">
      <div className="max-w-7xl mx-auto space-y-16">
        <div className="text-center space-y-4 max-w-3xl mx-auto">
          <div className="inline-flex items-center gap-2 px-3.5 py-1 rounded-full bg-slate-900 border border-slate-800 text-xs font-mono text-cyan-400">
            <Sparkles className="w-3.5 h-3.5" />
            <span>Core Pillars</span>
          </div>
          <h2 className="font-['Space_Grotesk'] text-3xl sm:text-4xl font-bold tracking-tight text-white">
            Engineered for Cloud Infrastructure
          </h2>
          <p className="text-slate-400 text-sm sm:text-base leading-relaxed">
            Dockworker eliminates container vulnerabilities, runtime mutations, and unpredictable build environments across enterprise swarms.
          </p>
        </div>

        <div className="grid md:grid-cols-2 gap-6">
          {features.map((f, i) => {
            const Icon = f.icon;
            return (
              <div
                key={i}
                className={`p-8 rounded-2xl bg-gradient-to-br ${f.color} border ${f.border} backdrop-blur-xl space-y-4 relative overflow-hidden transition-transform hover:-translate-y-1`}
              >
                <div className="flex items-center justify-between">
                  <div className={`w-12 h-12 rounded-xl bg-slate-900/80 border border-slate-700/80 flex items-center justify-center ${f.iconColor}`}>
                    <Icon className="w-6 h-6" />
                  </div>
                  <span className="text-[11px] font-mono px-3 py-1 rounded-full bg-slate-900/80 text-slate-300 border border-slate-800">
                    {f.badge}
                  </span>
                </div>
                <h3 className="text-xl font-bold text-white font-['Space_Grotesk']">{f.title}</h3>
                <p className="text-slate-300 text-sm leading-relaxed">{f.desc}</p>
              </div>
            );
          })}
        </div>
      </div>
    </section>
  );
}
