import type {
  CohortComparisonResult,
  DeletePreview,
  DeleteTarget,
  HistoryFilter,
  HistoryItemSummary,
  PeerTrendPoint,
  SessionRecord,
} from "../../lib/contracts/history";
import {
  compareCohort,
  deleteConfirm,
  deletePreview,
  getHistoryDetail,
  getPeerTrend,
  listHistory,
} from "../../lib/api/history";

export class HistoryModel {
  items = $state<HistoryItemSummary[]>([]);
  totalCount = $state(0);
  hasMore = $state(false);
  isLoading = $state(false);

  // Filtros
  searchQuery = $state("");
  selectedPeerId = $state<string | null>(null);
  selectedVerdict = $state<string | null>(null);
  selectedProtocol = $state<string | null>(null);

  // Sesión abierta en detalle
  selectedSession = $state<SessionRecord | null>(null);
  selectedSessionId = $state<string | null>(null);
  cohortComparison = $state<CohortComparisonResult | null>(null);

  // Evolución / Tendencia por equipo
  peerTrend = $state<PeerTrendPoint[]>([]);

  // Modal de confirmación de borrado
  deleteModal = $state<{
    isOpen: boolean;
    target: DeleteTarget | null;
    preview: DeletePreview | null;
    isDeleting: boolean;
    error: string | null;
  }>({
    isOpen: false,
    target: null,
    preview: null,
    isDeleting: false,
    error: null,
  });

  async load() {
    this.isLoading = true;
    try {
      const filter: HistoryFilter = {
        searchQuery: this.searchQuery.trim() ? this.searchQuery.trim() : null,
        peerId: this.selectedPeerId,
        verdict: this.selectedVerdict,
        protocol: this.selectedProtocol,
      };

      const result = await listHistory(filter, { limit: 50, offset: 0 });
      this.items = result.items;
      this.totalCount = result.totalCount;
      this.hasMore = result.hasMore;

      // Si hay un peer seleccionado, cargar tendencia
      if (this.selectedPeerId) {
        this.peerTrend = await getPeerTrend(
          this.selectedPeerId,
          this.selectedProtocol ?? undefined,
        );
      } else {
        this.peerTrend = [];
      }
    } catch {
      this.items = [];
      this.totalCount = 0;
      this.hasMore = false;
    } finally {
      this.isLoading = false;
    }
  }

  async selectSession(sessionId: string) {
    this.selectedSessionId = sessionId;
    try {
      const detail = await getHistoryDetail(sessionId);
      this.selectedSession = detail;
      this.cohortComparison = await compareCohort(sessionId);
    } catch {
      this.selectedSession = null;
      this.cohortComparison = null;
    }
  }

  closeDetail() {
    this.selectedSession = null;
    this.selectedSessionId = null;
    this.cohortComparison = null;
  }

  async promptDelete(target: DeleteTarget) {
    this.deleteModal.isOpen = true;
    this.deleteModal.target = target;
    this.deleteModal.error = null;
    try {
      const preview = await deletePreview(target);
      this.deleteModal.preview = preview;
    } catch {
      this.deleteModal.preview = null;
      this.deleteModal.error = "No se pudo obtener la previsualización del borrado";
    }
  }

  async executeDelete(): Promise<boolean> {
    if (!this.deleteModal.preview) return false;
    this.deleteModal.isDeleting = true;
    try {
      const result = await deleteConfirm(this.deleteModal.preview.confirmationToken);
      if (result) {
        this.deleteModal.isOpen = false;
        this.deleteModal.preview = null;
        this.deleteModal.target = null;
        if (this.selectedSessionId) {
          this.closeDetail();
        }
        await this.load();
        return true;
      }
    } catch {
      this.deleteModal.error = "Error al ejecutar el borrado transaccional";
    } finally {
      this.deleteModal.isDeleting = false;
    }
    return false;
  }

  cancelDelete() {
    this.deleteModal.isOpen = false;
    this.deleteModal.target = null;
    this.deleteModal.preview = null;
    this.deleteModal.isDeleting = false;
    this.deleteModal.error = null;
  }
}
