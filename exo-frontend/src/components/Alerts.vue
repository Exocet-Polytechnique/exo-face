<template>
    <div id="container">
        <h1 id="title">Alertes</h1>
        <table id="alerts-table">
            <thead>
                <tr id="alerts-table-header">
                    <th>PCB</th>
                    <th>Alerte</th>
                </tr>
            </thead>
            <tbody>
                <tr v-for="pcb in data" :key="pcb.name" :class="rowClass(pcb.alert)">
                    <td>{{ pcb.name }}</td>
                    <td>{{ pcb.alert ? pcb.alert.title : 'Aucune alerte' }}</td>
                </tr>
            </tbody>
        </table>
    </div>
</template>

<script>

export default {
  name: 'AlertsComponent',
  props: {
    data: {
        type: Array,
        required: true
    }
  },
  methods: {
    rowClass(alert) {
      return alert ? alert.severity : '';
    }
  }
}
</script>

<style>
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

#alerts-table {
  width: 100%;
  border-collapse: collapse;
}

#alerts-table th {
  padding-top: 0.3vh;
  padding-left: 0.3vh;
  padding-bottom: 0.3vh;
  text-align: left;
  color: white;
  font-size: 5vh;
}

#alerts-table-header {
    background-color: #c12736;
}

.warning {
    background-color: #f7e379;
}
.error {
    background-color: #e51f1f;
    color: white;
    animation: blink-error 1s step-start infinite;
}

@keyframes blink-error {
    50% {
        background-color: #7a0000;
    }
}

#alerts-table td {
  padding-top: 0.3vh;
  padding-left: 0.3vh;
  padding-bottom: 0.3vh;
  text-align: left;
  font-size: 5vh;
}
</style>
