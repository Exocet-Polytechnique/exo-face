<template>
    <div id="container">
        <h1 id="title">État du Bateau</h1>
        <span id="label" :class="stateClass">{{ stateText }}</span>
    </div>
</template>

<script>

export default {
  name: 'BoatStateComponent',
  props: {
    state: {
        type: Number,
        required: true
    }
  },
  computed: {
    stateText() {
        switch (this.state) {
            case 0: return 'Arrêté';
            case 1: return 'Démarrage...';
            case 2: return 'En Marche';
            case 3: return 'Arrêt en cours...';
            default: return 'État Inconnu';
        }
    },
    stateClass() {
        switch (this.state) {
            case 0: return 'idle';
            case 1: return 'transition';
            case 2: return 'started';
            case 3: return 'transition';
            default: return 'idle';
        }
    }
  }
}
</script>

<style scoped>

#container {
    display: flex;
    flex-direction: row;
    justify-content: space-between;
    align-items: center;
    height: 100%;
}

#title {
    margin: 0.5vh;
    padding: 0;
    text-align: left;
    font-size: 6vh;
    color: #313239;
}

#label {
    margin: 0.5vh;
    font-size: 6vh;
    font-weight: 700;
    padding: 0vh 8vh 0vh 0vh;
}

.transition {
    color: #d4a017;
}

.started {
    color: #2e9e12;
}

</style>
