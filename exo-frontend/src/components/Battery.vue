<template>
  <div id="container">
    <h1 id="title">Batteries</h1>
    <table id="battery-table">
      <thead>
        <tr id="battery-table-header">
          <th>Module</th>
          <th>Statut</th>
          <th>Dur. Est.</th>
          <th>Voltage(V)</th>
          <th>Courant(A)</th>
          <th>Temp.(°C)</th>
        </tr>
      </thead>
      <tbody>
        <tr 
          v-for="row in data" 
          :key="row.id" 
          :class="{ danger: modulesInDanger.includes(row.id)}"
        >
          <td>{{ row.displayName || row.id }}</td>
          <td>{{ row.status }}</td>
          <td>{{ row.duration }}</td>
          <td>{{ row.voltage }}</td>
          <td>{{ row.current }}</td>
          
          <!-- Section Température avec surlignage dynamique -->
          <td :class="['temp-cell', getAlertClass(row)]">
            <div class="temp-wrapper">
              <span class="temp-text">{{ row.temperature }}</span>
            </div>
          </td>
        </tr>
      </tbody>
    </table>

    <div id="fuel-cells-section">
      <div class="section-content">
        <table id="fuel-cells-table">
          <thead>
            <tr id="fuel-cells-table-header">
              <th>Fuel Cell</th>
              <th>Tension (V)</th>
              <th>Puissance (kW)</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="cell in fuelCells" :key="cell.id">
              <td>{{ cell.name }}</td>
              <td>{{ cell.tension }}</td>
              <td>{{ cell.puissance }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>

<script>

export default {
  name: 'BatteryComponent',
  props: {
    data: {
        type: Array,
        required: true
    },
    modulesInDanger:{
        type: Array,
        required: false,
      default: () => []
    }
  },
  data() {
    return {
      fuelCells: [
        { id: 1, name: 'Fuel Cell 1', tension: '48.00', puissance: '12.00' },
        { id: 2, name: 'Fuel Cell 2', tension: '47.50', puissance: '11.50' }
      ]
    };
  },
  methods: {
    /**
     * Calcule la classe d'alerte pour le fond de la cellule
     */
    getAlertClass(row) {
      const t = parseFloat(row.temperature);
      const max = row.maxTemp || 65; 
      const ratio = t / max;

      if (ratio >= 0.95) return 'critical';
      if (ratio >= 0.90) return 'alert';
      if (ratio >= 0.70) return 'warning';
      return '';
    }
  }
}
</script>

<style scoped>
#container {
  display: flex;
  flex-direction: column;
  height: 100%;
}
#title {
    margin: 0.5vh;
    padding: 0;
    text-align: left;
    font-size: 6vh;
    color: #313239;
}

#battery-table,
#fuel-cells-table {
  width: 100%;
  border-collapse: collapse;
}

#battery-table th,
#fuel-cells-table th {
  padding: 0.3vh;
  text-align: left;
  color: white;
  font-size: 5vh;
}

#battery-table-header,
#fuel-cells-table-header {
    background-color: #c12736;
}

#battery-table td,
#fuel-cells-table td { 
  padding: 0.3vh; 
  text-align: left; 
  font-size: 5vh; 
}


.danger {
    background-color: #fe7e44;
}

.temp-cell {
  transition: background-color 0.3s ease;
}

.temp-wrapper {
  display: flex;
  align-items: center;
  width: 100%;
  height: 100%;
}

.warning { 
  background-color: #ffeb3b; 
  color: #000;
}

.alert { 
  background-color: #ff9800; 
  color: #fff; 
}

.critical { 
  background-color: #f44336; 
  color: #fff;
  animation: pulse-bg-red 1.2s infinite; 
}

#fuel-cells-section {
  margin-top: 1.5vh;
}

.section-header {
  background-color: #c12736;
  color: #fff;
  padding: 0.8vh 1vh;
  border-radius: 8px;
  font-size: 4.5vh;
  font-weight: 700;
  margin-bottom: 0.8vh;
}

.section-content {
  background-color: #f7f7f7;
  padding: 1vh;
  border-radius: 8px;
  min-height: 8vh;
}

@keyframes pulse-bg-red {
  0% { background-color: #f44336; }
  50% { background-color: #b71c1c; }
  100% { background-color: #f44336; }
}
</style>