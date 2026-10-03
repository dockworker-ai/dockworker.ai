import React from 'react';
import { GitPullRequest, CheckCircle2, Shield, Lock, Server, Cloud } from 'lucide-react';

export function ArchitectureSection() {
  return (
    <section id="zero-dockerfile" className="py-24 px-6 lg:px-12 border-t border-slate-900">
      <div className="max-w-7xl mx-auto space-y-16">
        <div className="grid lg:grid-cols-2 gap-12 items-center">
          <div className="space-y-6">
            <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-emerald-950/40 border border-emerald-500/30 text-xs font-mono text-emerald-300">
              <Shield className="w-3.5 h-3.5" />
              <span>Security Baseline</span>
            </div>
            <h2 className="font-['Space_Grotesk'] text-3xl sm:text-4xl font-bold tracking-tight text-white">
              Why Swarms Reject Dockerfiles
            </h2>
            <div className="space-y-4 text-slate-300 text-sm sm:text-base leading-relaxed">
              <p>
                Legacy Dockerfiles introduce non-deterministic state, unpinned package managers (<code className="text-cyan-300 font-mono text-xs">apt-get update</code>), and unverified base image supply chains.
              </p>
              <p>
                Under the **Lornu AI & AIVCS Swarm Standard**, every layer is declared in pure Nix expressions with fixed SHA-256 output hashes. Container images compile bit-for-bit identical across any developer workstation, local sandbox, or central Propel build daemon.
              </p>
            </div>

            <div className="space-y-3 pt-2">
              <div className="flex items-start gap-3">
                <CheckCircle2 className="w-5 h-5 text-cyan-400 shrink-0 mt-0.5" />
                <span className="text-sm text-slate-200">Zero root daemon or Docker socket exposures during packaging</span>
              </div>
              <div className="flex items-start gap-3">
                <CheckCircle2 className="w-5 h-5 text-cyan-400 shrink-0 mt-0.5" />
                <span className="text-sm text-slate-200">Automated Content-Addressable Storage (CAS) layer deduplication</span>
              </div>
              <div className="flex items-start gap-3">
                <CheckCircle2 className="w-5 h-5 text-cyan-400 shrink-0 mt-0.5" />
                <span className="text-sm text-slate-200">GCP Workload Identity Federation (WIF) OIDC authentication</span>
              </div>
            </div>
          </div>

          <div className="rounded-2xl bg-slate-900/60 border border-slate-800 p-8 space-y-6">
            <h3 className="text-lg font-bold text-white font-['Space_Grotesk'] flex items-center gap-2">
              <Server className="w-5 h-5 text-cyan-400" />
              Build Flow Matrix
            </h3>
            
            <div className="space-y-4 font-mono text-xs">
              <div className="p-4 rounded-xl bg-slate-950/80 border border-slate-800 space-y-1">
                <div className="text-slate-400">1. Developer / Agent Commit:</div>
                <div className="text-cyan-300">aivcs push --repo=aivcs/&lt;app&gt;</div>
              </div>
              <div className="p-4 rounded-xl bg-slate-950/80 border border-slate-800 space-y-1">
                <div className="text-slate-400">2. Propel Webhook Daemon:</div>
                <div className="text-emerald-300">oci-dockworker-build-propel compile --nix-pure</div>
              </div>
              <div className="p-4 rounded-xl bg-slate-950/80 border border-slate-800 space-y-1">
                <div className="text-slate-400">3. Registry & Edge Distribution:</div>
                <div className="text-purple-300">GAR / ECR OCI Push + Cloudflare Pages Edge</div>
              </div>
              <div className="p-4 rounded-xl bg-slate-950/80 border border-slate-800 space-y-1">
                <div className="text-slate-400">4. OxidizedFlux Reconciliation:</div>
                <div className="text-amber-300">aivcsops reconcile --live --zero-drift</div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
